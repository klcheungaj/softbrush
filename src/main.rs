use mimalloc::MiMalloc;
use softbrush_ls::lsp::Backend;
use std::io::{self, Write};
use std::process::ExitCode;
use tower_lsp::{LspService, Server};

#[cfg(debug_assertions)]
mod debug_cli;

#[global_allocator]
static G_ALLOCATOR: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    if arguments.len() == 1 && arguments[0] == "--version" {
        return print_version();
    }
    if arguments.len() == 1 && matches!(arguments[0].to_str(), Some("--help" | "-h")) {
        return print_help();
    }
    if arguments.is_empty() || arguments.as_slice() == ["--stdio"] {
        serve();
        return ExitCode::SUCCESS;
    }

    #[cfg(debug_assertions)]
    return debug_cli::run(&arguments);
    #[cfg(not(debug_assertions))]
    {
        eprintln!(
            "softbrush_ls: unknown argument; run without arguments or with --stdio for LSP stdio"
        );
        ExitCode::from(2)
    }
}

fn print_help() -> ExitCode {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    match write_help(&mut output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("softbrush_ls: {error}");
            ExitCode::from(2)
        }
    }
}

fn print_version() -> ExitCode {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    match writeln!(output, "{}", env!("CARGO_PKG_VERSION")) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("softbrush_ls: {error}");
            ExitCode::from(2)
        }
    }
}

fn write_help(output: &mut impl Write) -> io::Result<()> {
    writeln!(output, "Usage: softbrush_ls [ARGUMENT]")?;
    writeln!(output)?;
    writeln!(output, "Supported arguments:")?;
    writeln!(output, "    (none)")?;
    writeln!(
        output,
        "        Start the language server over stdin/stdout."
    )?;
    writeln!(output, "    --stdio")?;
    writeln!(
        output,
        "        Start the language server over stdin/stdout."
    )?;
    writeln!(output, "    --help, -h")?;
    writeln!(output, "        Print this help message and exit.")?;
    writeln!(output, "    --version")?;
    writeln!(output, "        Print the version number and exit.")?;
    #[cfg(debug_assertions)]
    {
        writeln!(output)?;
        writeln!(output, "    --dump-tokens [--] FILE...")?;
        writeln!(
            output,
            "        Dump lexer tokens, parsed words, semantic tokens, unclassified text, and diagnostics."
        )?;
        writeln!(
            output,
            "        FILE must have a .tcl, .sdc, or .xdc extension; use -- before a filename beginning with '-'."
        )?;
        writeln!(
            output,
            "        This argument is available only in debug builds."
        )?;
    }
    Ok(())
}

#[tokio::main]
async fn serve() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
