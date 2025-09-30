use lolg::Lolg;
use std::{collections::hash_map::IntoKeys, sync::Arc};
use tokio::sync::{Mutex, broadcast, watch};

use crate::{
    game::{
        enums::Action,
        game_action::GameAction,
        match_manager::{MatchManager, MatchStatus},
        player::Player,
    },
    network::{client::Client, setup::Setup},
    protocol::packet::{Packet, PacketKind},
    utils::{
        errors::Error,
        models::{Discard, JoinRequest},
        types::ClientPool,
    },
};

pub struct Protocol {
    logger: Arc<Lolg>,
    client_pool: ClientPool, // Connected client pool (Also held by ClientManager).
    global_id: Arc<Mutex<i32>>, // Id tracker for packets sent by server
    mmrx: watch::Receiver<MatchStatus>, // Watches the MatchStatus that is sent from MatchManager.
    pub bctx: broadcast::Sender<Packet>, // Global packet broadcaster to all clients.
    pub match_manager: Arc<MatchManager>,
}

// PUBLIC METHODS
impl Protocol {
    pub async fn new(log_manager: Arc<Lolg>, client_pool: ClientPool) -> Result<Arc<Self>, Error> {
        let (bctx, _rx) = broadcast::channel::<Packet>(4);
        let (mmtx, mmrx) = watch::channel(MatchStatus::Waiting);
        let match_manager = MatchManager::new(log_manager.clone(), mmtx).await?;

        let protocol = Arc::new(Self {
            mmrx,
            bctx,
            client_pool,
            logger: log_manager,
            global_id: Arc::new(Mutex::new(0)),
            match_manager: Arc::new(match_manager),
        });

        Arc::clone(&protocol).watch_match_status().await;
        return Ok(protocol);
    }

    // Spawns a task to handle a packet based on its PacketKind and currently directly sends a response.
    pub async fn handle_packet(self: Arc<Self>, client: Arc<Client>, packet: Packet) {
        tokio::spawn(async move {
            match packet.kind {
                PacketKind::Setup => self.handle_setup(client, &packet).await,
                PacketKind::Action => self.handle_action(client.clone(), &packet).await,
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
        let req = JoinRequest::parse(&packet.body[5..])?;
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
        return JoinRequest::parse(&packet.body[5..]);
    }
}

impl Protocol {
    async fn handle_action(&self, client: Arc<Client>, p: &Packet) {
        match GameAction::parse(&p.body) {
            Err(error) => {
                let addr = client.addr.read().await;
                self.logger.error(&format!("{addr}: {error}")).await;
                let packet = Packet::error(p.id, error);
                client.send_packet(&packet).await;
            }
            Ok(action) => {
                let player = Arc::clone(&client.player);
                match action.action {
                    Action::DRAW => {
                        let response = match self.match_manager.draw(player).await {
                            Err(error) => {
                                self.logger.error(&error.to_string()).await;
                                Packet::error(p.id, error)
                            }
                            Ok(tile) => match serde_cbor::to_vec(&tile) {
                                Err(_) => {
                                    let error = Error::InternalError;
                                    self.logger.error(&error.to_string()).await;
                                    Packet::error(p.id, error)
                                }
                                Ok(bytes) => {
                                    let mut body = Vec::new();
                                    body.extend_from_slice(&Action::DRAW.bytes());
                                    body.extend_from_slice(&bytes);
                                    Packet::create(
                                        p.id,
                                        PacketKind::Action,
                                        &body.into_boxed_slice(),
                                    )
                                }
                            },
                        };

                        client.send_packet(&response).await;
                    }
                    Action::KAN => {
                        todo!()
                    }
                    Action::CHI => {
                        todo!()
                    }
                    Action::PON => {
                        todo!()
                    }
                    Action::RON => {
                        todo!()
                    }
                    Action::TSUMO => {
                        todo!()
                    }
                    Action::DISCARD => {
                        match self.match_manager.discard(player, action).await {
                            Err(error) => {
                                let response = Packet::error(p.id, error);
                                client.send_packet(&response).await;
                            }
                            Ok(tile) => {
                                let Ok(melds) = self.match_manager.check_calls(tile).await else {
                                    let id = self.get_global_id().await;
                                    let error = Error::InternalError;
                                    let response = Packet::error(id, error);
                                    let _ = self.bctx.send(response);
                                    return;
                                };

                                let client_pool = self.client_pool.read().await;
                                for key in melds.keys() {
                                    let client = client_pool.get(key).unwrap();
                                    let meld = melds.get(key).unwrap();

                                    let id = self.get_global_id().await;
                                    let mut body = Vec::new();
                                    body.extend_from_slice(&Action::DISCARD.bytes());
                                    body.extend_from_slice(&meld);
                                    let response = Packet::create(id, PacketKind::Action, &body);
                                    client.send_packet(&response).await;
                                }
                            }
                        };
                    }
                };
            }
        };
    }

    async fn handle_setup(&self, client: Arc<Client>, packet: &Packet) {
        let Some(operation) = Setup::from(&packet.body[..4]) else {
            let error = Error::ConnectionNeeded;
            let addr = client.addr.read().await;
            self.logger.error(&format!("{addr}: {error}")).await;
            let _ = client.send_packet(&Packet::error(packet.id, error)).await;
            return;
        };

        let response = match operation {
            Setup::Initialization => match client.player.get_initial_view().await {
                Ok(view_bytes) => {
                    let setup = Setup::Initialization.bytes();
                    let mut body_bytes = setup.to_vec();
                    body_bytes.extend(view_bytes);
                    Packet::create(packet.id, PacketKind::Setup, &body_bytes)
                }
                Err(error) => {
                    let addr = client.addr.read().await;
                    self.logger.error(&format!("{addr}: {error}")).await;
                    Packet::error(packet.id, error)
                }
            },
            Setup::Ready => {
                client.player.set_ready().await;
                let addr = client.addr.read().await;
                self.logger.info(&format!("{addr}: is ready.")).await;
                Packet::create(packet.id, PacketKind::Setup, &[0x00])
            }
            _ => {
                let error = Error::OperationFailed(57);
                let addr = client.addr.read().await;
                self.logger.error(&format!("{addr}: {error}")).await;
                Packet::error(packet.id, error)
            }
        };

        client.send_packet(&response).await;
    }

    // Spawns a task to watch the changes from the match status and deal with each respective status.
    async fn watch_match_status(self: Arc<Self>) {
        let bctx = self.bctx.clone();
        let mut mmrx = self.mmrx.clone();
        tokio::spawn({
            async move {
                loop {
                    let status = *mmrx.borrow();
                    match &status {
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
