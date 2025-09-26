use crate::network::server::Server;

mod game;
mod network;
mod protocol;
mod utils;

#[tokio::main]
async fn main() {
    let server = Server::create_instance(3000)
        .await
        .expect("Server initialization failed");
    server.start().await;
}
