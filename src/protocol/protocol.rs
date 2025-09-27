use lolg::Lolg;
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
    utils::{errors::Error, models::JoinRequest, types::ClientPool},
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
    pub async fn new(log_manager: Arc<Lolg>, client_pool: ClientPool) -> Arc<Self> {
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
                    Action::KAN => { todo!() }
                    Action::CHI => { todo!() }
                    Action::PON => { todo!() }
                    Action::RON => { todo!() }
                    Action::TSUMO => { todo!() }
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

    async fn handle_ping(&self, client: Arc<Client>, packet: &Packet) {
        let pong = "Pong!".as_bytes();
        let response = Packet::create(packet.id, PacketKind::Ping, pong);
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
                        MatchStatus::Ready => {
                            let kind = PacketKind::MatchStatus;
                            let body = MatchStatus::Ready.bytes();
                            let mut id = self.global_id.lock().await;
                            let packet = Packet::create(*id, kind, &body);
                            *id += 1;
                            drop(id);
                            let _ = bctx.send(packet);
                            self.logger.debug(&format!("STATUS CHANGE: {status}")).await;
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
