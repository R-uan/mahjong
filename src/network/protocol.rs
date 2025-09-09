use tokio::sync::RwLock;

use crate::{
    game::{game_state::GameManager, player::Player},
    network::client::Client,
    protocol::packet::{Packet, PacketType},
};
use std::sync::Arc;

pub struct Protocol {
    global_id: Arc<RwLock<i32>>,
    game_manager: Arc<GameManager>,
}

impl Protocol {
    pub fn new(gm: Arc<GameManager>) -> Self {
        Self {
            game_manager: gm,
            global_id: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn handle_packet(&self, client: Arc<Client>, packet: Packet) {
        let response: Option<Packet> = match packet.packet_type {
            PacketType::Ping => Some(self.handle_ping(&packet)),
            PacketType::Reconnection => {}
            PacketType::Authentication => {}
            PacketType::GameAction => {}
        };

        if let Some(packet) = response {
            client.send_packet(packet).await;
        }
    }

    async fn handle_game_action(&self) {}

    fn handle_ping(&self, incoming: &Packet) -> Packet {
        let pong = "Pong!".as_bytes();
        let packet = Packet::create(incoming.packet_id, PacketType::Ping, pong);
        return packet;
    }

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
