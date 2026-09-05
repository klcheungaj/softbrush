//! Debug-only token reports. No token classification is performed here.

use std::io::{self, Write};
use std::ops::Range;
use std::path::Path;

use antlr4_runtime::{CommonTokenStream, InputStream, Token as _};

use super::{Document, TOKEN_TYPES, encode_semantic_tokens};
use crate::analysis::{LineIndex, analyze};
use crate::catalog::Dialect;
use crate::generated::tcl_lexer::{METADATA, TclLexer};

/// Writes a deterministic report for a single Tcl, SDC, or XDC file.
///
/// Reports raw lexer tokens (including whitespace and EOF), parsed words,
/// the actual LSP semantic tokens, unclassified gaps, and diagnostics.
/// Locations are zero-based UTF-16; byte ranges are half-open UTF-8.
/// Source syntax errors are report data and do not fail the command.
///
/// # Errors
/// Returns an error for unsupported extensions, file reads (including invalid
/// UTF-8), non-regular files, or output failures.
#[allow(
    clippy::unnecessary_debug_formatting,
    reason = "Escape tabs, newlines, and non-UTF-8 bytes in report filenames"
)]
pub fn write_report(path: &Path, output: &mut impl Write) -> io::Result<()> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if !["tcl", "sdc", "xdc"]
        .iter()
        .any(|value| extension.eq_ignore_ascii_case(value))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a .tcl, .sdc, or .xdc file",
        ));
    }
    if !path.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a regular file",
        ));
    }
    let source = std::fs::read_to_string(path)?;
    let dialect = Dialect::from_uri_path(&format!("input.{extension}"));
    let document = Document {
        index: LineIndex::new(&source),
        analysis: analyze(&source, dialect),
        text: source,
        dialect,
    };
    writeln!(
        output,
        "# softbrush.tokenDump/v1 file={:?} dialect={dialect:?}",
        path.as_os_str()
    )?;
    writeln!(
        output,
        "# layer\tbytes\tutf16(start-end)\tkind\tmetadata\ttext (Rust debug escaped)"
    )?;
    write_lexer(&document, output)?;
    let mut commands = document.analysis.syntax.commands.iter().collect::<Vec<_>>();
    commands.sort_by_key(|command| (command.span.start, command.span.end));
    for command in commands {
        for (index, word) in command.words.iter().enumerate() {
            row(
                output,
                &document,
                "word",
                word.span.clone(),
                if index == 0 { "command" } else { "argument" },
                &format!("depth={}", command.nesting),
            )?;
        }
    }
    write_semantics(&document, output)?;
    for diagnostic in &document.analysis.diagnostics {
        row(
            output,
            &document,
            "diagnostic",
            diagnostic.span.clone(),
            diagnostic.code,
            &format!("{:?}: {:?}", diagnostic.severity, diagnostic.message),
        )?;
    }
    writeln!(
        output,
        "# summary commands={} semantic_spans={} diagnostics={} antlr_errors={}",
        document.analysis.syntax.commands.len(),
        document.analysis.semantic_spans.len(),
        document.analysis.diagnostics.len(),
        document.analysis.syntax.antlr_syntax_errors
    )
}

fn write_lexer(document: &Document, output: &mut impl Write) -> io::Result<()> {
    let mut lexer = CommonTokenStream::new(TclLexer::new(InputStream::new(&document.text)));
    lexer.fill();
    let vocabulary = METADATA.vocabulary();
    for token in lexer.tokens() {
        let span = if token.token_type() == antlr4_runtime::TOKEN_EOF {
            document.text.len()..document.text.len()
        } else {
            token
                .byte_span()
                .ok_or_else(|| io::Error::other("lexer token has no source span"))?
        };
        row(
            output,
            document,
            "lexer",
            span,
            &vocabulary.display_name(token.token_type()),
            &format!("channel={}", token.channel()),
        )?;
    }
    Ok(())
}

fn write_semantics(document: &Document, output: &mut impl Write) -> io::Result<()> {
    let mut line = 0;
    let mut column = 0;
    let mut cursor = 0;
    for token in encode_semantic_tokens(document) {
        line += token.delta_line;
        column = if token.delta_line == 0 {
            column + token.delta_start
        } else {
            token.delta_start
        };
        let start = document.index.offset(line, column);
        let end = document.index.offset(line, column + token.length);
        if cursor < start {
            row(
                output,
                document,
                "semantic",
                cursor..start,
                "unclassified",
                "-",
            )?;
        }
        row(
            output,
            document,
            "semantic",
            start..end,
            TOKEN_TYPES[token.token_type as usize].as_str(),
            if token.token_modifiers_bitset == 0 {
                "-"
            } else {
                "declaration"
            },
        )?;
        cursor = end;
    }
    if cursor < document.text.len() {
        row(
            output,
            document,
            "semantic",
            cursor..document.text.len(),
            "unclassified",
            "-",
        )?;
    }
    Ok(())
}

fn row(
    output: &mut impl Write,
    document: &Document,
    layer: &str,
    span: Range<usize>,
    kind: &str,
    metadata: &str,
) -> io::Result<()> {
    let (line, column) = document.index.position(span.start);
    let (end_line, end_column) = document.index.position(span.end);
    writeln!(
        output,
        "{layer}\t{}..{}\t{line}:{column}-{end_line}:{end_column}\t{kind}\t{metadata}\t{:?}",
        span.start,
        span.end,
        &document.text[span.clone()]
    )
}
