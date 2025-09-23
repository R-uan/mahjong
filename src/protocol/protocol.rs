use std::sync::Arc;
use tokio::sync::{Mutex, broadcast, watch};

use crate::{
    game::{
        game_action::{Action, GameAction},
        match_manager::{MatchManager, MatchStatus},
        player::Player,
    },
    network::{client::Client, setup::Setup},
    protocol::packet::{Packet, PacketKind},
    utils::{errors::Error, log_manager::LogManager, models::JoinRequest, types::ClientPool},
};

pub struct Protocol {
    logger: Arc<LogManager>,
    client_pool: ClientPool, // Connected client pool (Also held by ClientManager).
    global_id: Arc<Mutex<i32>>, // Id tracker for packets sent by server
    mmrx: watch::Receiver<MatchStatus>, // Watches the MatchStatus that is sent from MatchManager.
    pub bctx: broadcast::Sender<Packet>, // Global packet broadcaster to all clients.
    pub match_manager: Arc<MatchManager>,
}

// PUBLIC METHODS
impl Protocol {
    pub async fn new(log_manager: Arc<LogManager>, client_pool: ClientPool) -> Arc<Self> {
        let (bctx, _rx) = broadcast::channel::<Packet>(4);
        let (mmtx, mmrx) = watch::channel(MatchStatus::Waiting);
        let match_manager = MatchManager::new(log_manager.clone(), mmtx);
    
        let protocol = Arc::new(Self {
            mmrx,
            bctx,
            client_pool,
            logger: log_manager,
            global_id: Arc::new(Mutex::new(0)),
            match_manager: Arc::new(match_manager),
        });
    
        Arc::clone(&protocol).watch_match_status().await;
        return protocol;
    }

    // Spawns a task to handle a packet based on its PacketKind and currently directly sends a response.
    pub async fn handle_packet(self: Arc<Self>, client: Arc<Client>, packet: Packet) {
        tokio::spawn(async move {
            match packet.kind {
                PacketKind::Ping => self.handle_ping(client, &packet).await,
                PacketKind::Pong => {}
                PacketKind::Setup => self.handle_setup(client, &packet).await,
                PacketKind::GameAction => self.handle_action(client.clone(), &packet).await,
                _ => {
                    let error = Error::PacketParsingFailed(102);
                    let packet = Packet::error(packet.id, error);
                    client.send_packet(&packet).await;
                }
            };
        });
    }

    // Handles packets of the Connection kind.
    pub async fn handle_connect(&self, packet: &Packet) -> Result<Arc<Player>, Error> {
        let req = serde_cbor::from_slice::<JoinRequest>(&packet.body)
            .map_err(|_| Error::ConnectionFailed(105))?;
        let player = self.match_manager.assign_player(&req).await?;
        return Ok(player);
    }

    pub async fn get_global_id(&self) -> i32 {
        let mut id = self.global_id.lock().await;
        *id += 1;
        return *id;
    }

    // Handles packets of the Reconnection kind.
    pub fn handle_reconnect(&self, packet: &Packet) -> Result<JoinRequest, Error> {
        return serde_cbor::from_slice::<JoinRequest>(&packet.body)
            .map_err(|_| Error::ConnectionFailed(105));
    }
}

impl Protocol {
    async fn handle_action(&self, client: Arc<Client>, p: &Packet) {
        match GameAction::parse(&p.body) {
            Err(error) => {
                let addr = client.addr.read().await;
                let log_msg = format!("{addr}: {}", &error.to_string());
                self.logger.error(log_msg, "PROTOC").await;
                let packet = Packet::error(p.id, error);
                client.send_packet(&packet).await;
            }
            Ok(action) => {
                let player = Arc::clone(&client.player);
                match action.action {
                    Action::KAN => {}
                    Action::CHI => {}
                    Action::TSUMO => {}
                    Action::PON => {}
                    Action::RON => {}
                    Action::DISCARD => {
                        let response = match self.match_manager.discard(player, action).await {
                            Err(error) => Packet::error(p.id, error),
                            Ok(result) => match result {
                                true => Packet::create(p.id, PacketKind::ActionDone, &p.body),
                                false => Packet::create(p.id, PacketKind::ActionFail, &p.body),
                            },
                        };

                        client.send_packet(&response).await;
                    }
                };
            }
        };
    }

    async fn handle_setup(&self, client: Arc<Client>, packet: &Packet) {
        let Some(operation) = Setup::from(&packet.body[..4]) else {
            let error = Error::ConnectionNeeded;
            let addr = client.addr.read().await;
            let log_error = format!("{addr}: {}", error.to_string());
            self.logger.error(log_error, "CM").await;
            let _ = client.send_packet(&Packet::error(packet.id, error)).await;
            return;
        };

        let response = match operation {
            Setup::Initialization => match client.player.get_initial_view().await {
                Ok(view_bytes) => Packet::create(packet.id, PacketKind::Setup, &view_bytes),
                Err(error) => {
                    let addr = client.addr.read().await;
                    let log_error = format!("{addr}: {}", error.to_string());
                    self.logger.error(log_error, "CM").await;
                    Packet::error(packet.id, error)
                }
            },
            Setup::Ready => {
                client.player.set_ready().await;
                let addr = client.addr.read().await;
                let log_msg = format!("{addr}: is ready.");
                self.logger.info(log_msg, "PROTOC").await;
                Packet::create(packet.id, PacketKind::Setup, &[0x00])
            }
            _ => {
                let error = Error::OperationFailed(57);
                let addr = client.addr.read().await;
                let log_error = format!("{addr}: {}", error.to_string());
                self.logger.error(log_error, "CM").await;
                Packet::error(packet.id, error)
            }
        };

        client.send_packet(&response).await;
    }

    async fn handle_ping(&self, client: Arc<Client>, packet: &Packet) {
        let pong = "Pong!".as_bytes();
        let response = Packet::create(packet.id, PacketKind::Ping, pong);
        client.send_packet(&response).await;
    }

    async fn handle_initialization(&self) {}

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
                            let mut id = self.global_id.lock().await;
                            let packet = Packet::create(*id, kind, &body);
                            *id += 1;
                            drop(id);
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
}
