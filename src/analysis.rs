//! Pure semantic analysis, diagnostics, symbols, and source-position mapping.

use std::ops::Range;

use crate::catalog::{self, Dialect, TCL_KEYWORDS};
use crate::syntax::{Command, SyntaxModel, Word, parse};

/// Severity assigned to a language-server diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    /// A definite error that prevents the construct from being valid.
    Error,
    /// A likely problem that may depend on tool-specific behavior.
    Warning,
    /// Informational guidance with no correctness implication.
    Information,
    /// A low-priority suggestion, including tolerant catalog mismatches.
    Hint,
}

/// A source diagnostic produced by syntax checking or linting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// Half-open UTF-8 byte range associated with the diagnostic.
    pub span: Range<usize>,
    /// Diagnostic severity.
    pub severity: Severity,
    /// Stable machine-readable diagnostic code.
    pub code: &'static str,
    /// Human-readable explanation.
    pub message: String,
}

/// Semantic token category independent of the LSP wire representation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SemanticKind {
    /// Tcl comment.
    Comment,
    /// Quoted or braced string.
    String,
    /// Numeric literal.
    Number,
    /// Variable reference or declaration.
    Variable,
    /// Command or procedure name.
    Function,
    /// Tcl control-flow keyword.
    Keyword,
    /// Command option or expression operator.
    Operator,
    /// Procedure parameter declaration.
    Parameter,
    /// Namespace declaration.
    Namespace,
}

/// A semantic classification over a source range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticSpan {
    /// Half-open UTF-8 byte range in the source document.
    pub span: Range<usize>,
    /// Semantic category.
    pub kind: SemanticKind,
    /// Whether the span declares its symbol.
    pub declaration: bool,
}

/// Symbol category independent of the LSP wire representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolKind {
    /// Tcl procedure.
    Function,
    /// Tcl variable.
    Variable,
    /// Tcl namespace.
    Namespace,
    /// SDC or XDC clock.
    Clock,
    /// Tool-specific design object.
    Object,
}

/// A document symbol extracted from a Tcl command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Symbol {
    /// Display name.
    pub name: String,
    /// Short category detail shown by clients.
    pub detail: &'static str,
    /// Symbol category.
    pub kind: SymbolKind,
    /// Half-open range of the defining command.
    pub span: Range<usize>,
    /// Half-open range of the symbol name.
    pub selection_span: Range<usize>,
}

/// Complete analysis result for one source document.
#[derive(Clone, Debug)]
pub struct Analysis {
    /// Tolerant syntax model.
    pub syntax: SyntaxModel,
    /// Syntax and lint diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// Semantic classifications used for highlighting.
    pub semantic_spans: Vec<SemanticSpan>,
    /// Symbols declared in the document.
    pub symbols: Vec<Symbol>,
}

/// Parses and analyzes source according to the selected Tcl-based dialect.
///
/// Constraint command catalogs are intentionally advisory: unknown SDC and XDC
/// commands produce hints because vendor extensions are common.
#[must_use]
pub fn analyze(source: &str, dialect: Dialect) -> Analysis {
    let syntax = parse(source);
    let mut diagnostics = syntax
        .errors
        .iter()
        .map(|error| Diagnostic {
            span: error.span.clone(),
            severity: Severity::Error,
            code: "tcl-syntax",
            message: error.message.clone(),
        })
        .collect::<Vec<_>>();

    lint_continuations(source, &mut diagnostics);
    for command in &syntax.commands {
        lint_command(command, dialect, &mut diagnostics);
    }

    let mut semantic_spans = Vec::new();
    semantic_spans.extend(syntax.comments.iter().cloned().map(|span| SemanticSpan {
        span,
        kind: SemanticKind::Comment,
        declaration: false,
    }));
    classify_strings(&syntax, &mut semantic_spans);
    semantic_spans.extend(syntax.variables.iter().cloned().map(|span| SemanticSpan {
        span,
        kind: SemanticKind::Variable,
        declaration: false,
    }));

    let mut symbols = Vec::new();
    for command in &syntax.commands {
        classify_command(command, &mut semantic_spans);
        extract_symbol(command, &mut symbols, &mut semantic_spans);
        classify_procedure_parameters(source, command, &mut semantic_spans);
    }
    classify_arguments(&syntax.commands, &mut semantic_spans);
    remove_overlapping_spans(&mut semantic_spans);

    diagnostics.sort_by_key(|diagnostic| (diagnostic.span.start, diagnostic.span.end));
    symbols.sort_by_key(|symbol| symbol.span.start);
    Analysis {
        syntax,
        diagnostics,
        semantic_spans,
        symbols,
    }
}

fn is_procedure_argument_list(commands: &[Command], span: &Range<usize>) -> bool {
    commands.iter().any(|command| {
        command.name() == Some("proc")
            && command.words.get(2).is_some_and(|word| word.span == *span)
    })
}

fn classify_strings(syntax: &SyntaxModel, spans: &mut Vec<SemanticSpan>) {
    for string in syntax
        .strings
        .iter()
        .filter(|span| !is_procedure_argument_list(&syntax.commands, span))
    {
        let mut segment_start = string.start;
        for variable in syntax
            .variables
            .iter()
            .filter(|variable| variable.start >= string.start && variable.end <= string.end)
        {
            if segment_start < variable.start {
                spans.push(SemanticSpan {
                    span: segment_start..variable.start,
                    kind: SemanticKind::String,
                    declaration: false,
                });
            }
            segment_start = segment_start.max(variable.end);
        }
        if segment_start < string.end {
            spans.push(SemanticSpan {
                span: segment_start..string.end,
                kind: SemanticKind::String,
                declaration: false,
            });
        }
    }
}

fn classify_procedure_parameters(source: &str, command: &Command, spans: &mut Vec<SemanticSpan>) {
    if command.name() != Some("proc") {
        return;
    }
    let Some(arguments) = command.words.get(2) else {
        return;
    };
    if !arguments.text.starts_with('{') || !arguments.text.ends_with('}') {
        return;
    }
    let Some(interior_end) = arguments.span.end.checked_sub(1) else {
        return;
    };
    let mut position = arguments.span.start + 1;
    while position < interior_end {
        position = skip_list_whitespace(source, position, interior_end);
        if position >= interior_end {
            break;
        }

        let current = source[position..]
            .chars()
            .next()
            .expect("position is in source");
        let (name_start, name_end, element_end) = if current == '{' {
            let element_end = find_braced_list_element_end(source, position, interior_end);
            let name_start = skip_list_whitespace(source, position + 1, element_end);
            let name_end = scan_list_name(source, name_start, element_end.saturating_sub(1));
            (name_start, name_end, element_end)
        } else {
            let element_end = scan_list_name(source, position, interior_end);
            (position, element_end, element_end)
        };

        if name_end > name_start {
            spans.push(SemanticSpan {
                span: name_start..name_end,
                kind: SemanticKind::Parameter,
                declaration: true,
            });
        }
        position = element_end.max(position + current.len_utf8());
    }
}

fn skip_list_whitespace(source: &str, mut position: usize, end: usize) -> usize {
    while position < end {
        let current = source[position..]
            .chars()
            .next()
            .expect("position is in source");
        if !current.is_whitespace() {
            break;
        }
        position += current.len_utf8();
    }
    position
}

fn scan_list_name(source: &str, mut position: usize, end: usize) -> usize {
    while position < end {
        let current = source[position..]
            .chars()
            .next()
            .expect("position is in source");
        if current.is_whitespace() || current == '}' {
            break;
        }
        position += current.len_utf8();
        if current == '\\' && position < end {
            position += source[position..]
                .chars()
                .next()
                .expect("position is in source")
                .len_utf8();
        }
    }
    position
}

fn find_braced_list_element_end(source: &str, start: usize, end: usize) -> usize {
    let mut position = start + 1;
    let mut depth = 1usize;
    while position < end {
        let current = source[position..]
            .chars()
            .next()
            .expect("position is in source");
        position += current.len_utf8();
        match current {
            '\\' if position < end => {
                position += source[position..]
                    .chars()
                    .next()
                    .expect("position is in source")
                    .len_utf8();
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return position;
                }
            }
            _ => {}
        }
    }
    end
}

fn lint_command(command: &Command, dialect: Dialect, diagnostics: &mut Vec<Diagnostic>) {
    let Some(name) = command.name() else {
        return;
    };
    let name_span = command.words[0].span.clone();

    match name {
        "proc" if command.words.len() < 4 => diagnostics.push(Diagnostic {
            span: command.span.clone(),
            severity: Severity::Error,
            code: "tcl-arity",
            message: "`proc` expects a name, argument list, and body".to_owned(),
        }),
        "set" if command.words.len() < 2 => diagnostics.push(Diagnostic {
            span: command.span.clone(),
            severity: Severity::Error,
            code: "tcl-arity",
            message: "`set` expects a variable name".to_owned(),
        }),
        "expr"
            if command
                .words
                .get(1)
                .is_some_and(|word| !word.text.starts_with('{')) =>
        {
            diagnostics.push(Diagnostic {
                span: command.words[1].span.clone(),
                severity: Severity::Hint,
                code: "tcl-unbraced-expr",
                message: "prefer a braced expression to avoid double substitution".to_owned(),
            });
        }
        "create_clock" => lint_create_clock(command, diagnostics),
        "create_generated_clock" => {
            require_option(command, "-source", Severity::Warning, diagnostics);
        }
        "set_input_delay" | "set_output_delay" => {
            require_option(command, "-clock", Severity::Warning, diagnostics);
        }
        "set_multicycle_path" if first_positional(command, 1).is_none() => {
            diagnostics.push(Diagnostic {
                span: command.span.clone(),
                severity: Severity::Warning,
                code: "constraint-missing-value",
                message: "`set_multicycle_path` is missing its cycle multiplier".to_owned(),
            });
        }
        "set_property" if dialect == Dialect::Xdc && command.words.len() < 4 => {
            diagnostics.push(Diagnostic {
                span: command.span.clone(),
                severity: Severity::Warning,
                code: "xdc-arity",
                message: "`set_property` normally expects a property, value, and object collection"
                    .to_owned(),
            });
        }
        _ => {}
    }

    if dialect != Dialect::Tcl && !catalog::is_known_command(dialect, name) {
        let suggestion = catalog::commands(dialect)
            .map(|candidate| (strsim::jaro_winkler(name, candidate), candidate))
            .filter(|(score, _)| *score >= 0.88)
            .max_by(|left, right| left.0.total_cmp(&right.0));
        let message = suggestion.map_or_else(
            || format!("`{name}` is not in the bundled command catalog; vendor extensions may still be valid"),
            |(_, candidate)| format!("unknown catalog command `{name}`; did you mean `{candidate}`?"),
        );
        diagnostics.push(Diagnostic {
            span: name_span,
            severity: Severity::Hint,
            code: "constraint-unknown-command",
            message,
        });
    }
}

fn lint_create_clock(command: &Command, diagnostics: &mut Vec<Diagnostic>) {
    let Some(period_index) = option_index(command, "-period") else {
        diagnostics.push(Diagnostic {
            span: command.span.clone(),
            severity: Severity::Warning,
            code: "constraint-missing-option",
            message: "`create_clock` normally requires `-period`".to_owned(),
        });
        return;
    };
    let Some(value) = command.words.get(period_index + 1) else {
        diagnostics.push(Diagnostic {
            span: command.words[period_index].span.clone(),
            severity: Severity::Error,
            code: "constraint-missing-value",
            message: "`-period` requires a value".to_owned(),
        });
        return;
    };
    if value
        .plain_text()
        .and_then(|text| text.parse::<f64>().ok())
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        diagnostics.push(Diagnostic {
            span: value.span.clone(),
            severity: Severity::Error,
            code: "constraint-invalid-value",
            message: "clock period must be finite and greater than zero".to_owned(),
        });
    }
}

fn require_option(
    command: &Command,
    option: &str,
    severity: Severity,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(index) = option_index(command, option) else {
        diagnostics.push(Diagnostic {
            span: command.span.clone(),
            severity,
            code: "constraint-missing-option",
            message: format!(
                "`{}` normally requires `{option}`",
                command.name().unwrap_or_default()
            ),
        });
        return;
    };
    if command
        .words
        .get(index + 1)
        .is_none_or(|word| word.text.starts_with('-'))
    {
        diagnostics.push(Diagnostic {
            span: command.words[index].span.clone(),
            severity: Severity::Error,
            code: "constraint-missing-value",
            message: format!("`{option}` requires a value"),
        });
    }
}

fn option_index(command: &Command, option: &str) -> Option<usize> {
    command.words.iter().position(|word| word.text == option)
}

fn first_positional(command: &Command, start: usize) -> Option<&Word> {
    command.words[start..]
        .iter()
        .find(|word| !word.text.starts_with('-'))
}

fn lint_continuations(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut line_start = 0;
    for line in source.split_inclusive('\n') {
        let without_lf = line.strip_suffix('\n').unwrap_or(line);
        let content = without_lf.strip_suffix('\r').unwrap_or(without_lf);
        let trimmed = content.trim_end_matches([' ', '\t']);
        if trimmed.ends_with('\\') && trimmed.len() != content.len() {
            let slash = line_start + trimmed.len() - 1;
            diagnostics.push(Diagnostic {
                span: slash..(line_start + content.len()),
                severity: Severity::Warning,
                code: "tcl-continuation-whitespace",
                message: "spaces after `\\` prevent Tcl line continuation".to_owned(),
            });
        }
        line_start += line.len();
    }
}

fn classify_command(command: &Command, spans: &mut Vec<SemanticSpan>) {
    let Some(name) = command.name() else {
        return;
    };
    spans.push(SemanticSpan {
        span: command.words[0].span.clone(),
        kind: if TCL_KEYWORDS.contains(&name) {
            SemanticKind::Keyword
        } else {
            SemanticKind::Function
        },
        declaration: false,
    });
}

fn classify_arguments(commands: &[Command], spans: &mut Vec<SemanticSpan>) {
    for command in commands {
        for word in command.words.iter().skip(1) {
            if word.text.starts_with('-') && word.plain_text().is_some() {
                spans.push(SemanticSpan {
                    span: word.span.clone(),
                    kind: SemanticKind::Operator,
                    declaration: false,
                });
            } else if word
                .plain_text()
                .and_then(|text| text.parse::<f64>().ok())
                .is_some()
            {
                spans.push(SemanticSpan {
                    span: word.span.clone(),
                    kind: SemanticKind::Number,
                    declaration: false,
                });
            }
        }
    }
}

fn extract_symbol(command: &Command, symbols: &mut Vec<Symbol>, spans: &mut Vec<SemanticSpan>) {
    let Some(name) = command.name() else {
        return;
    };
    let (word, kind, detail, semantic_kind) = match name {
        "proc" => (
            command.words.get(1),
            SymbolKind::Function,
            "Tcl procedure",
            SemanticKind::Function,
        ),
        "namespace" if command.words.get(1).is_some_and(|word| word.text == "eval") => (
            command.words.get(2),
            SymbolKind::Namespace,
            "Tcl namespace",
            SemanticKind::Namespace,
        ),
        "set" if command.words.len() >= 3 => (
            command.words.get(1),
            SymbolKind::Variable,
            "Tcl variable",
            SemanticKind::Variable,
        ),
        "variable" => (
            command.words.get(1),
            SymbolKind::Variable,
            "Tcl variable",
            SemanticKind::Variable,
        ),
        "create_clock" | "create_generated_clock" => (
            named_or_last_word(command, "-name"),
            SymbolKind::Clock,
            "Timing clock",
            SemanticKind::Variable,
        ),
        "create_pblock" | "create_macro" | "create_debug_core" => (
            command.words.get(1),
            SymbolKind::Object,
            "XDC object",
            SemanticKind::Variable,
        ),
        _ => return,
    };
    let Some(word) = word else {
        return;
    };
    let symbol_name = word.unquoted_text();
    if symbol_name.is_empty() || symbol_name.starts_with(['-', '[', '$']) {
        return;
    }
    symbols.push(Symbol {
        name: symbol_name.to_owned(),
        detail,
        kind,
        span: command.span.clone(),
        selection_span: word.span.clone(),
    });
    spans.push(SemanticSpan {
        span: word.span.clone(),
        kind: semantic_kind,
        declaration: true,
    });
}

fn named_or_last_word<'a>(command: &'a Command, option: &str) -> Option<&'a Word> {
    option_index(command, option)
        .and_then(|index| command.words.get(index + 1))
        .or_else(|| {
            command
                .words
                .last()
                .filter(|word| !word.text.starts_with('-'))
        })
}

fn remove_overlapping_spans(spans: &mut Vec<SemanticSpan>) {
    spans.sort_by_key(|span| {
        (
            span.span.start,
            std::cmp::Reverse(span.span.end),
            span.declaration,
        )
    });
    let mut occupied_until = 0;
    spans.retain(|span| {
        if span.span.is_empty() || span.span.start < occupied_until {
            return false;
        }
        occupied_until = span.span.end;
        true
    });
    spans.sort_by_key(|span| span.span.start);
}

/// Converts between UTF-8 byte offsets and zero-based LSP UTF-16 positions.
#[derive(Clone, Debug)]
pub struct LineIndex {
    source: String,
    line_starts: Vec<usize>,
}

impl LineIndex {
    /// Builds an index for a source snapshot.
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(source.match_indices('\n').map(|(offset, _)| offset + 1));
        Self {
            source: source.to_owned(),
            line_starts,
        }
    }

    /// Converts a UTF-8 byte offset into `(line, UTF-16 column)`.
    ///
    /// Offsets past the source are clamped to the end. An offset inside a
    /// multibyte UTF-8 character is clamped to that character's start.
    #[must_use]
    pub fn position(&self, byte_offset: usize) -> (u32, u32) {
        let mut offset = byte_offset.min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset = offset.saturating_sub(1);
        }
        let line = self
            .line_starts
            .partition_point(|start| *start <= offset)
            .saturating_sub(1);
        let offset = offset.min(self.line_content_end(line));
        let character = self.source[self.line_starts[line]..offset]
            .encode_utf16()
            .count();
        (
            u32::try_from(line).unwrap_or(u32::MAX),
            u32::try_from(character).unwrap_or(u32::MAX),
        )
    }

    /// Converts an LSP UTF-16 position to a UTF-8 byte offset.
    ///
    /// Missing lines map to the end of the source. Columns past a line are
    /// clamped to the line content and never cross its newline.
    #[must_use]
    pub fn offset(&self, line: u32, utf16_character: u32) -> usize {
        let line = line as usize;
        let Some(&start) = self.line_starts.get(line) else {
            return self.source.len();
        };
        let end = self.line_content_end(line);
        let mut units = 0u32;
        for (relative, character) in self.source[start..end].char_indices() {
            if units >= utf16_character {
                return start + relative;
            }
            let character_units = u32::try_from(character.len_utf16()).unwrap_or(2);
            if units.saturating_add(character_units) > utf16_character {
                return start + relative;
            }
            units += character_units;
        }
        end
    }

    /// Returns the exclusive UTF-8 byte end of a line, excluding CR/LF bytes.
    #[must_use]
    pub fn line_end(&self, line: u32) -> usize {
        let line = line as usize;
        if self.line_starts.get(line).is_none() {
            return self.source.len();
        }
        self.line_content_end(line)
    }

    /// Returns the UTF-8 byte offset at which a line starts.
    #[must_use]
    pub fn line_start(&self, line: u32) -> usize {
        self.line_starts
            .get(line as usize)
            .copied()
            .unwrap_or(self.source.len())
    }

    fn line_content_end(&self, line: usize) -> usize {
        let start = self.line_starts[line];
        let mut end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.source.len());
        if end > start && self.source.as_bytes()[end - 1] == b'\n' {
            end -= 1;
        }
        if end > start && self.source.as_bytes()[end - 1] == b'\r' {
            end -= 1;
        }
        end
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::Dialect;

    use super::{LineIndex, Severity, SymbolKind, analyze};

    #[test]
    fn warns_about_invalid_clock_period() {
        let analysis = analyze(
            "create_clock -name sys -period 0 [get_ports clk]\n",
            Dialect::Sdc,
        );
        assert!(analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "constraint-invalid-value" && diagnostic.severity == Severity::Error
        }));
    }

    #[test]
    fn extracts_procedure_and_clock_symbols() {
        let analysis = analyze(
            "proc twice {x} {expr {$x * 2}}\ncreate_clock -name sys -period 10 [get_ports clk]\n",
            Dialect::Sdc,
        );
        assert!(
            analysis
                .symbols
                .iter()
                .any(|symbol| symbol.name == "twice" && symbol.kind == SymbolKind::Function)
        );
        assert!(
            analysis
                .symbols
                .iter()
                .any(|symbol| symbol.name == "sys" && symbol.kind == SymbolKind::Clock)
        );
    }

    #[test]
    fn treats_unknown_sdc_commands_as_non_blocking_hints() {
        let analysis = analyze(
            "create_clok -period 10\nacme_vendor_constraint foo\n",
            Dialect::Sdc,
        );
        let unknown = analysis
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "constraint-unknown-command")
            .collect::<Vec<_>>();
        assert_eq!(unknown.len(), 2);
        assert!(
            unknown
                .iter()
                .all(|diagnostic| diagnostic.severity == Severity::Hint)
        );
        assert!(unknown[0].message.contains("create_clock"));
    }

    #[test]
    fn line_index_uses_utf16_columns() {
        let index = LineIndex::new("a😀b\nnext");

        assert_eq!(index.position(1), (0, 1));
        assert_eq!(index.position(2), (0, 1));
        assert_eq!(index.position("a😀".len()), (0, 3));
        assert_eq!(index.offset(0, 1), 1);
        assert_eq!(index.offset(0, 2), 1);
        assert_eq!(index.offset(0, 3), "a😀".len());
    }

    #[test]
    fn line_index_clamps_invalid_positions_to_content_boundaries() {
        let source = "a😀b\r\nlast";
        let index = LineIndex::new(source);

        assert_eq!(index.position(2), (0, 1));
        assert_eq!(index.offset(0, 2), 1);
        assert_eq!(index.offset(0, u32::MAX), "a😀b".len());
        assert_eq!(index.line_end(0), "a😀b".len());
        assert_eq!(index.line_start(1), "a😀b\r\n".len());
        assert_eq!(index.position("a😀b".len()), (0, 4));
        assert_eq!(index.position("a😀b\r".len()), (0, 4));
        assert_eq!(index.position("a😀b\r\n".len()), (1, 0));
        assert_eq!(index.offset(0, 4), "a😀b".len());
        assert_eq!(index.offset(0, 5), "a😀b".len());
        assert_eq!(index.line_end(1), source.len());
        assert_eq!(index.offset(1, u32::MAX), source.len());
        assert_eq!(index.position(source.len()), (1, 4));
    }

    #[test]
    fn line_index_handles_empty_and_trailing_newline_sources() {
        let empty = LineIndex::new("");
        assert_eq!(empty.position(1), (0, 0));
        assert_eq!(empty.offset(1, 1), 0);
        assert_eq!(empty.line_end(0), 0);

        let trailing_newline = LineIndex::new("a\n");
        assert_eq!(trailing_newline.position(2), (1, 0));
        assert_eq!(trailing_newline.line_start(1), 2);
        assert_eq!(trailing_newline.line_end(1), 2);
    }
}
