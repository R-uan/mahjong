use crate::network::client::Client;
use crate::network::setup::Setup;
use crate::protocol::packet::{Packet, PacketKind, WriteBytesExt};
use crate::protocol::protocol::Protocol;
use crate::utils::errors::Error;
use crate::utils::types::ClientPool;
use lolg::Lolg;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

pub struct ClientManager {
    pub logger: Arc<Lolg>,
    pub protocol: Arc<Protocol>,
    pub client_pool: ClientPool,
}

impl ClientManager {
    pub async fn new(logger: Arc<Lolg>) -> Result<Self, Error> {
        let client_pool: ClientPool = Arc::new(RwLock::new(HashMap::default()));
        let protocol = Protocol::new(Arc::clone(&logger), Arc::clone(&client_pool)).await?;
        Ok(Self {
            logger,
            protocol,
            client_pool: Arc::new(RwLock::new(HashMap::default())),
        })
    }

    // Handles the initial client state (unauthenticated)
    // Client has ~~five~~ one attempt~~s~~ to send a connection packet.
    // Create a Client struct when/if successfully authenticated.
    // Store client and run the main listen loop for the definitive Client.
    pub async fn accept(self: Arc<Self>, mut stream: TcpStream, addr: SocketAddr) {
        tokio::spawn(async move {
            let mut attempts = 0;
            let mut buffer = [0; 1024];

            while attempts < 5 {
                let read_bytes = match stream.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => {
                        attempts += 1;
                        continue;
                    }
                };

                let _ = match Packet::from_bytes(&buffer[..read_bytes]) {
                    Err(error) => {
                        self.logger.error(&format!("{addr}: {error}")).await;
                        let id = self.protocol.get_global_id().await;
                        let _ = stream.send_packet(&Packet::error(id, error)).await;
                    }
                    Ok(packet) => {
                        if packet.kind == PacketKind::Setup {
                            let Some(operation) = Setup::from(&packet.body[..4]) else {
                                let error = Error::ConnectionNeeded;
                                self.logger.error(&format!("{addr}: {error}")).await;
                                let _ = stream.send_packet(&Packet::error(packet.id, error)).await;
                                return;
                            };

                            match operation {
                                Setup::Connection => {
                                    match self.protocol.handle_connect(&packet).await {
                                        Err(error) => {
                                            self.logger.error(&format!("{addr}: {error}")).await;
                                            let response = Packet::error(packet.id, error);
                                            let _ = stream.send_packet(&response).await;
                                        }
                                        Ok(player) => {
                                            let id = player.id;
                                            let protocol = self.protocol.clone();
                                            let bcrx = protocol.bctx.subscribe();
                                            {
                                                let alias = &player.alias.read().await;
                                                let log_msg =
                                                    format!("{addr}: connected as {alias}");
                                                self.logger.info(&log_msg).await;
                                            }
                                            let client = Client::new(
                                                id, addr, stream, player, protocol, bcrx,
                                            )
                                            .await;

                                            Arc::clone(&client).connect().await;
                                            self.client_pool.write().await.insert(id, client);
                                            self.protocol.match_manager.check_ready().await;
                                            return;
                                        }
                                    }
                                }
                                Setup::Reconnection => {
                                    match self.protocol.handle_reconnect(&packet) {
                                        Err(error) => {
                                            self.logger.error(&format!("{addr}: {error}")).await;
                                            let response = Packet::error(packet.id, error);
                                            let _ = stream.send_packet(&response).await;
                                        }
                                        Ok(request) => {
                                            match self.client_pool.read().await.get(&request.id) {
                                                None => {
                                                    let error = Error::ReconnectionFailed(55);
                                                    self.logger
                                                        .error(&format!("{addr}: {error}"))
                                                        .await;
                                                    let response = Packet::error(packet.id, error);
                                                    let _ = stream.send_packet(&response).await;
                                                }
                                                Some(client) => {
                                                    self.logger
                                                        .info(&format!("{addr}: reconnected"))
                                                        .await;
                                                    Arc::clone(&client)
                                                        .reconnect(stream, addr)
                                                        .await;
                                                    return;
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    let error = Error::ConnectionNeeded;
                                    self.logger.error(&format!("{addr}: {error}")).await;
                                    let response = Packet::error(packet.id, error);
                                    let _ = stream.send_packet(&response).await;
                                    return;
                                }
                            };
                        } else {
                            let error = Error::ConnectionNeeded;
                            self.logger.error(&format!("{addr}: {error}")).await;
                            let _ = stream.send_packet(&Packet::error(packet.id, error)).await;
                            return;
                        }
                    }
                };

                attempts += 1;
            }
        });
    }
}
