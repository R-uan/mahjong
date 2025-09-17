use tokio::sync::RwLock;

use crate::{
    game::{
        game_action::{Action, GameAction},
        game_state::GameManager,
        player::Player,
    },
    network::client::Client,
    protocol::packet::{Packet, PacketKind},
    utils::{errors::Error, log_manager::LogManager, models::AuthResponse},
};
use std::sync::Arc;

pub struct Protocol {
    pub logger: Arc<LogManager>,
    game_manager: Arc<GameManager>,
}

impl Protocol {
    pub fn new(gm: Arc<GameManager>, lm: Arc<LogManager>) -> Self {
        Self {
            logger: lm,
            game_manager: gm,
        }
    }

    pub async fn handle_packet(&self, client: Arc<Client>, packet: Packet) {
        let response: Option<Packet> = match packet.kind {
            PacketKind::Ping => Some(self.handle_ping(&packet)),
            PacketKind::GameAction => self.handle_action(client.clone(), &packet).await,
            _ => Some(Packet::create(packet.id, PacketKind::Error, "".as_bytes())),
        };

        if let Some(packet) = response {
            client.send_packet(packet).await;
        }
    }

    async fn handle_action(&self, client: Arc<Client>, p: &Packet) -> Option<Packet> {
        match GameAction::parse(&p.body) {
            Ok(action) => {
                let player = Arc::clone(&client.player);
                match action.action {
                    Action::DISCARD => match self.game_manager.discard(player, action).await {
                        Err(error) => Some(Packet::create(
                            p.id,
                            PacketKind::Error,
                            error.to_string().as_bytes(),
                        )),
                        Ok(discard) => match discard {
                            true => Some(Packet::create(p.id, PacketKind::ActionDone, &p.body)),
                            false => Some(Packet::create(p.id, PacketKind::ActionFail, &p.body)),
                        },
                    },
                    Action::KAN => None,
                    Action::CHI => None,
                    Action::TSUMO => None,
                    Action::PON => None,
                    Action::RON => None,
                }
            }
            Err(error) => None,
        }
    }

    fn handle_ping(&self, packet: &Packet) -> Packet {
        let pong = "Pong!".as_bytes();
        return Packet::create(packet.id, PacketKind::Ping, pong);
    }

    pub async fn connect_player(&self, auth: &AuthResponse) -> Result<Arc<Player>, Error> {
        return self.game_manager.assign_player(&auth.id, &auth.alias).await;
    }
}
