use crate::{network::server::Server, utils::errors::Error};

mod game;
mod network;
mod protocol;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let server = Server::create_instance(3000).await?;
    server.start().await;

    Ok(())
}
