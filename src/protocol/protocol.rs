use std::sync::Arc;
use tokio::sync::{broadcast, watch};

use crate::{
    game::{
        game_action::{Action, GameAction},
        match_manager::{MatchManager, MatchStatus},
        player::Player,
    },
    network::client::Client,
    protocol::packet::{Packet, PacketKind},
    utils::{errors::Error, log_manager::LogManager, models::JoinRequest, types::ClientPool},
};

pub struct Protocol {
    // Log manager for logging
    logger: Arc<LogManager>,
    // Connected client pool (Also held by ClientManager).
    // Used to update clients on the match status.
    client_pool: ClientPool,
    // Watches the MatchStatus that is sent from MatchManager.
    mmrx: watch::Receiver<MatchStatus>,
    pub match_manager: Arc<MatchManager>,
    // Global packet broadcaster to all clients.
    // Avoids the necessity to loop through the ClientPool.
    pub bctx: broadcast::Sender<Packet>,
}

impl Protocol {
    // Principal way to initialize an Protocol instance.
    pub async fn new(log_manager: Arc<LogManager>, client_pool: ClientPool) -> Arc<Self> {
        let (bctx, _rx) = broadcast::channel::<Packet>(4);
        let (mmtx, mmrx) = watch::channel(MatchStatus::Waiting);
        let match_manager = MatchManager::new(log_manager.clone(), mmtx);

        let protocol = Arc::new(Self {
            mmrx,
            bctx,
            client_pool,
            logger: log_manager,
            match_manager: Arc::new(match_manager),
        });

        Arc::clone(&protocol).watch_match_status().await;
        return protocol;
    }

    // Spawns a task to watch the changes from the match status and deal with each respective status.
    async fn watch_match_status(self: Arc<Self>) {
        let bctx = self.bctx.clone();
        let mut mmrx = self.mmrx.clone();
        tokio::spawn({
            async move {
                loop {
                    let match_status = *mmrx.borrow();

                    let log_msg = format!("MATCH STATUS CHANGE: {}", &match_status);
                    self.logger.debug(log_msg, "PROT").await;

                    match &match_status {
                        MatchStatus::Ready => {
                            let kind = PacketKind::MatchStatus;
                            let body = MatchStatus::Ready.bytes();
                            let packet = Packet::create(0, kind, &body);
                            let _ = bctx.send(packet);
                        }
                        MatchStatus::Waiting => {}
                        MatchStatus::Finished => {}
                        MatchStatus::Ongoing => {}
                        MatchStatus::Interrupted => {}
                    }

                    mmrx.changed().await.unwrap();
                }
            }
        });
    }

    // Spawns a task to handle a packet based on its PacketKind and currently directly sends a response.
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

    // Handles packets of the GameAction kind.
    async fn handle_action(&self, client: Arc<Client>, p: &Packet) -> Option<Packet> {
        match GameAction::parse(&p.body) {
            Err(_) => None,
            Ok(action) => {
                let player = Arc::clone(&client.player);
                match action.action {
                    Action::KAN => None,
                    Action::CHI => None,
                    Action::TSUMO => None,
                    Action::PON => None,
                    Action::RON => None,
                    Action::DISCARD => match self.match_manager.discard(player, action).await {
                        Err(error) => Some(Packet::error(p.id, error)),
                        Ok(result) => match result {
                            true => Some(Packet::create(p.id, PacketKind::ActionDone, &p.body)),
                            false => Some(Packet::create(p.id, PacketKind::ActionFail, &p.body)),
                        },
                    },
                }
            }
        }
    }

    // Handles packets of the Ping kind.
    fn handle_ping(&self, packet: &Packet) -> Packet {
        let pong = "Pong!".as_bytes();
        return Packet::create(packet.id, PacketKind::Ping, pong);
    }

    // Handles packets of the Connection kind.
    pub async fn handle_connect(&self, packet: &Packet) -> Result<Arc<Player>, Error> {
        let req = serde_cbor::from_slice::<JoinRequest>(&packet.body)
            .map_err(|_| Error::AuthenticationFailed(105))?;
        let player = self
            .match_manager
            .assign_player(&req.id, &req.alias)
            .await?;

        return Ok(player);
    }

    // Handles packets of the Reconnection kind.
    pub fn handle_reconnect(&self, packet: &Packet) -> Result<JoinRequest, Error> {
        return serde_cbor::from_slice::<JoinRequest>(&packet.body)
            .map_err(|_| Error::AuthenticationFailed(105));
    }
}
