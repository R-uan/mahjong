use crate::network::client::Client;
use crate::protocol::packet::{Packet, PacketKind, WriteBytesExt};
use crate::protocol::protocol::Protocol;
use crate::utils::errors::Error;
use crate::utils::log_manager::LogManager;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

pub struct ClientManager {
    pub logger: Arc<LogManager>,
    pub protocol: Arc<Protocol>,
    pub client_pool: Arc<RwLock<HashMap<String, Arc<Client>>>>,
}

impl ClientManager {
    pub fn new_manager(protocol: Arc<Protocol>, logger: Arc<LogManager>) -> Self {
        Self {
            logger,
            protocol,
            client_pool: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    // Handles the initial client state (unauthenticated)
    // Client has ~~five~~ one attempt~~s~~ to send a connection packet.
    // Create a Client struct when/if successfully authenticated.
    // Store client and run the main listen loop for the definitive Client.
    pub async fn accept(&self, mut stream: TcpStream, addr: SocketAddr) {
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
                Err(error) => stream.send_packet(&Packet::error(0, error)).await,
                Ok(packet) => match packet.kind {
                    //
                    // CONNECTION
                    PacketKind::Connection => match self.protocol.handle_connect(&packet).await {
                        Err(error) => stream.send_packet(&Packet::error(packet.id, error)).await,
                        Ok(player) => {
                            let protocol = self.protocol.clone();
                            let id = player.id.read().await.to_owned();
                            let client =
                                Client::new(id.to_owned(), addr, stream, player, protocol).await;
                            Arc::clone(&client).connect().await;
                            self.client_pool.write().await.insert(id, client);
                            return;
                        }
                    },
                    //
                    // RECONNECTION
                    PacketKind::Reconnection => match self.protocol.handle_reconnect(&packet) {
                        Err(error) => stream.send_packet(&Packet::error(packet.id, error)).await,
                        Ok(request) => match self.client_pool.read().await.get(&request.id) {
                            None => {
                                stream
                                    .send_packet(&Packet::error(
                                        packet.id,
                                        Error::ReconnectionFailed(55),
                                    ))
                                    .await
                            }
                            Some(client) => {
                                Arc::clone(&client).reconnect(stream, addr).await;
                                return;
                            }
                        },
                    },
                    _ => {
                        stream
                            .send_packet(&Packet::error(packet.id, Error::ConnectionNeeded))
                            .await
                    }
                },
            };

            attempts += 1;
        }
    }
}
