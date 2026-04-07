mod backend;

use backend::GObjectBackend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(GObjectBackend::new);
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}
