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
    if !arguments.is_empty() {
        #[cfg(debug_assertions)]
        return debug_cli::run(&arguments);
        #[cfg(not(debug_assertions))]
        {
            eprintln!(
                "softbrush_ls: release builds accept no arguments; run without arguments for LSP stdio"
            );
            return ExitCode::from(2);
        }
    }
    serve();
    ExitCode::SUCCESS
}

#[tokio::main]
async fn serve() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
