use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};

use crate::{
    game::{
        game_action::{Action, GameAction},
        match_manager::{MatchManager, MatchStatus},
        player::Player,
    },
    network::client::Client,
    protocol::packet::{Packet, PacketKind},
    utils::{errors::Error, log_manager::LogManager, models::JoinRequest},
};

pub struct Protocol {
    logger: Arc<LogManager>,
    match_manager: Arc<MatchManager>,
    bctx: Arc<RwLock<broadcast::Sender<Packet>>>,
    bcrx: Arc<RwLock<broadcast::Receiver<Packet>>>,
    mmch: Arc<RwLock<mpsc::Receiver<MatchStatus>>>,
}

impl Protocol {
    pub fn new(log_manager: Arc<LogManager>) -> Self {
        let (sender, receiver) = mpsc::channel(5);
        let (bctx, bcrx) = broadcast::channel::<Packet>(4);

        let match_manager = MatchManager::new_instance(log_manager.clone(), sender);
        Self {
            logger: log_manager,
            bcrx: Arc::new(RwLock::new(bcrx)),
            bctx: Arc::new(RwLock::new(bctx)),
            mmch: Arc::new(RwLock::new(receiver)),
            match_manager: Arc::new(match_manager),
        }
    }

    async fn handle_game_state(self: Arc<Self>) {
        let self_clone = Arc::clone(&self);
        tokio::spawn(async move {
            let match_status = self_clone.match_manager.status.read().await;
            while *match_status == MatchStatus::Ongoing {}
        });
    }

    async fn handle_match_status(self: Arc<Self>) {
        let self_clone = Arc::clone(&self);
        let match_clone = Arc::clone(&self.match_manager);
        tokio::spawn(async move {
            while *match_clone.status.read().await == MatchStatus::Ongoing {
                let mut mmch = self.mmch.write().await;
                if let Some(status) = mmch.recv().await {
                    todo!()
                }
            }
        });
    }

    pub async fn handle_packet(self: Arc<Self>, client: Arc<Client>, packet: Packet) {
        tokio::spawn(async move {
            let response: Option<Packet> = match packet.kind {
                PacketKind::Ping => Some(self.handle_ping(&packet)),
                PacketKind::GameAction => self.handle_action(client.clone(), &packet).await,
                _ => Some(Packet::create(packet.id, PacketKind::Error, "".as_bytes())),
            };

            if let Some(packet) = response {
                client.send_packet(packet).await;
            }
        });
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
