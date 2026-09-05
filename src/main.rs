use mimalloc::MiMalloc;
use softbrush_ls::lsp::Backend;
use std::process::ExitCode;
use tower_lsp::{LspService, Server};

#[cfg(debug_assertions)]
mod debug_cli;

#[global_allocator]
static G_ALLOCATOR: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
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

#[tokio::main]
async fn serve() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
