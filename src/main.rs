use std::sync::Arc;
use crate::game::game_state::GameState;
use crate::network::protocol::Protocol;
use crate::network::server::Server;

mod network;
mod protocol;
mod game;
#[tokio::main]
async fn main() {
    let game_state = GameState::start_game();
    if let Some(server_instance) = Server::create_instance(3000).await {
        let protocol = Arc::new(Protocol::new());
        
    }
    println!("Hello, world!");
}
