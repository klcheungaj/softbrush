//! Offline debug CLI. Compiled only with debug assertions enabled.

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "Usage: softbrush_ls --dump-tokens [--] FILE...\n\
    Dump lexer tokens, parsed words, semantic tokens, unclassified text, and diagnostics.\n\
    Supported files: .tcl, .sdc, .xdc (case insensitive). Debug builds only.\n\
    With no arguments or --stdio, serve LSP over stdio.";

pub(super) fn run(arguments: &[OsString]) -> ExitCode {
    if arguments[0] != "--dump-tokens" {
        eprintln!("softbrush_ls: unknown argument\n{USAGE}");
        return ExitCode::from(2);
    }
    let mut files = &arguments[1..];
    if files.first().is_some_and(|argument| argument == "--") {
        files = &files[1..];
    } else if files
        .iter()
        .any(|argument| argument.to_string_lossy().starts_with('-'))
    {
        eprintln!("softbrush_ls: use -- before filenames beginning with '-'\n{USAGE}");
        return ExitCode::from(2);
    }
    if files.is_empty() {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let mut output = io::BufWriter::new(io::stdout().lock());
    for file in files {
        if let Err(error) =
            softbrush_ls::lsp::debug_dump::write_report(Path::new(file), &mut output)
        {
            eprintln!("softbrush_ls: {}: {error}", Path::new(file).display());
            return ExitCode::from(2);
        }
    }
    match output.flush() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => failure(&error),
    }
}

fn failure(error: &io::Error) -> ExitCode {
    eprintln!("softbrush_ls: {error}");
    ExitCode::from(2)
}
