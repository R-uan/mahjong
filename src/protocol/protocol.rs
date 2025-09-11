use tokio::sync::RwLock;

use crate::{
    game::{errors::GameError, game_state::GameManager, player::Player},
    network::client::Client,
    protocol::{
        errors::ProtocolError,
        packet::{Packet, PacketType},
    },
    utils::models::AuthResponse,
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
            PacketType::Reconnection => None,
            PacketType::Authentication => None,
            PacketType::GameAction => self.handle_game_action(&packet).await,
        };

        if let Some(packet) = response {
            client.send_packet(packet).await;
        }
    }

    async fn handle_game_action(&self, packet: &Packet) -> Option<Packet> {
        todo!()
    }

    fn handle_ping(&self, packet: &Packet) -> Packet {
        let pong = "Pong!".as_bytes();
        return Packet::create(packet.packet_id, PacketType::Ping, pong);
    }

    pub async fn connect_player(&self, auth: &AuthResponse) -> Result<Arc<Player>, GameError> {
        return self.game_manager.assign_player(&auth.id, &auth.alias).await;
    }
}
