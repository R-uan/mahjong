use tokio::sync::RwLock;

use crate::{
    game::{errors::GameError, game_action::GameAction, game_state::GameManager, player::Player},
    network::client::Client,
    protocol::packet::{Packet, PacketType},
    utils::{log_manager::LogManager, models::AuthResponse},
};
use std::sync::Arc;

pub struct Protocol {
    pub log: Arc<LogManager>,
    global_id: Arc<RwLock<i32>>,
    game_manager: Arc<GameManager>,
}

impl Protocol {
    pub fn new(gm: Arc<GameManager>, lm: Arc<LogManager>) -> Self {
        Self {
            log: lm,
            game_manager: gm,
            global_id: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn handle_packet(&self, client: Arc<Client>, packet: Packet) {
        let response: Option<Packet> = match packet.packet_type {
            PacketType::Ping => Some(self.handle_ping(&packet)),
            PacketType::Reconnection => None,
            PacketType::Authentication => None,
            PacketType::GameAction => self.handle_game_action(client.clone(), &packet).await,
        };

        if let Some(packet) = response {
            client.send_packet(packet).await;
        }
    }

    async fn handle_game_action(&self, c: Arc<Client>, p: &Packet) -> Option<Packet> {
        match GameAction::parse(&p.packet_body) {
            Ok(action) => {
                let player = Arc::clone(&c.player);
                self.game_manager.apply_actions(player, action).await;
            }
            Err(error) => {}
        };
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
