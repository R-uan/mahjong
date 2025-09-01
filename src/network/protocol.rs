use crate::{
    game::{game_state::GameManager, player::Player},
    network::client::Client,
    protocol::packet::Packet,
};
use std::sync::Arc;

pub struct Protocol {
    game_manager: Arc<GameManager>,
}

impl Protocol {
    pub fn new(gm: Arc<GameManager>) -> Self {
        Self { game_manager: gm }
    }

    pub async fn handle_packet(&self, client: Arc<Client>, packet: Packet) {
        todo!()
    }

    async fn handle_game_action(&self) {}

    pub async fn assign_player(&self, id: &str, username: &str) -> Option<Arc<Player>> {
        match self.game_manager.get_free_seat().await {
            Some(seat) => {
                *seat.connected.write().await = true;
                *seat.id.write().await = id.to_string();
                *seat.alias.write().await = username.to_string();
                return Some(seat);
            }
            None => None,
        }
    }
}
