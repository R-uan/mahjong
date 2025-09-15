use crate::network::server::Server;
use std::sync::Arc;

mod game;
mod network;
mod protocol;
mod utils;

#[tokio::main]
async fn main() {
    if let Ok(server) = Server::create_instance(3000).await {
        let server_arc = Arc::new(server);
        server_arc.start().await;
    }
}
