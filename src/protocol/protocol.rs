use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver};

use crate::{
    game::{
        game_action::{Action, GameAction},
        match_manager::{MatchManager, MatchState},
        player::Player,
    },
    network::client::Client,
    protocol::packet::{Packet, PacketKind},
    utils::{errors::Error, log_manager::LogManager, models::JoinRequest},
};

pub struct Protocol {
    pub logger: Arc<LogManager>,
    match_manager: Arc<MatchManager>,
    match_manager_ch: Receiver<MatchState>,
}

impl Protocol {
    pub fn new(log_manager: Arc<LogManager>) -> Self {
        let (sender, receiver) = mpsc::channel(5);
        let match_manager = MatchManager::new_instance(log_manager.clone(), sender);
        Self {
            logger: log_manager,
            match_manager_ch: receiver,
            match_manager: Arc::new(match_manager),
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
            Err(_) => None,
            Ok(action) => {
                let player = Arc::clone(&client.player);
                match action.action {
                    Action::DISCARD => match self.match_manager.discard(player, action).await {
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
        }
    }

    fn handle_ping(&self, packet: &Packet) -> Packet {
        let pong = "Pong!".as_bytes();
        return Packet::create(packet.id, PacketKind::Ping, pong);
    }

    pub async fn handle_connect(&self, packet: &Packet) -> Result<Arc<Player>, Error> {
        let req = serde_cbor::from_slice::<JoinRequest>(&packet.body)
            .map_err(|_| Error::AuthenticationFailed(105))?;
        let player = self
            .match_manager
            .assign_player(&req.id, &req.alias)
            .await?;
        return Ok(player);
    }

    pub fn handle_reconnect(&self, packet: &Packet) -> Result<JoinRequest, Error> {
        return serde_cbor::from_slice::<JoinRequest>(&packet.body)
            .map_err(|_| Error::AuthenticationFailed(105));
    }
}
