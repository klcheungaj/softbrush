use mimalloc::MiMalloc;
use softbrush_ls::lsp::Backend;
use tower_lsp::{LspService, Server};

#[global_allocator]
static G_ALLOCATOR: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
