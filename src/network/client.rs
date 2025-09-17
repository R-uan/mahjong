use crate::game::player::Player;
use crate::protocol::packet::{Packet, PacketKind};
use crate::protocol::protocol::Protocol;
use crate::utils::errors::Error;
use crate::utils::log_manager::{LogLevel, LogManager};
use crate::utils::models::AuthResponse;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;
use tokio::time::sleep;

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
    // Client has five attempts to send an authentication packet.
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

            if let Ok(packet) = Packet::from_bytes(&buffer[..read_bytes]) {
                if packet.kind == PacketKind::Authentication {
                    if let Ok((player, auth)) = self.authenticate(&packet).await {
                        let mut client_poll = self.client_pool.write().await;
                        *player.connected.write().await = true;

                        self.logger
                            .send(
                                LogLevel::Info,
                                &format!("{addr} connected as {}", &player.alias.read().await),
                                "Client Manager",
                            )
                            .await;

                        let client = Client::new(
                            addr,
                            stream,
                            player,
                            self.protocol.clone(),
                            self.logger.clone(),
                        )
                        .await;

                        tokio::spawn({
                            let client_clone = Arc::clone(&client);
                            async move {
                                client_clone.connect().await;
                            }
                        });

                        let id = auth.id.to_string();
                        client_poll.insert(id, client);
                        return;
                    }
                } else if packet.kind == PacketKind::Reconnection {
                    if let Ok((_, auth)) = self.authenticate(&packet).await {
                        let client_guard = self.client_pool.read().await;
                        if let Some(client) = client_guard.get(&auth.id) {
                            let client_clone = Arc::clone(&client);
                            client_clone.reconnect(stream, addr).await;
                            return;
                        }
                    }
                }
            }

            attempts += 1;
        }
    }

    // Calls into the player auth server to authenticate player.
    pub async fn authenticate(&self, _: &Packet) -> Result<(Arc<Player>, AuthResponse), Error> {
        match reqwest::get("").await {
            Ok(response) => match response.status() {
                StatusCode::OK => {
                    let auth = response
                        .json::<AuthResponse>()
                        .await
                        .map_err(|_| Error::AuthenticationFailed(6))?;
                    let player = self
                        .protocol
                        .connect_player(&auth)
                        .await
                        .map_err(|_| Error::MatchAlreadyFull)?;
                    return Ok((player, auth));
                }
                _ => Err(Error::AuthenticationFailed(5)),
            },
            Err(_) => Err(Error::AuthenticationFailed(5)),
        }
    }
}

pub struct Client {
    pub player: Arc<Player>,
    pub logger: Arc<LogManager>,
    pub protocol: Arc<Protocol>,
    pub listening: Arc<RwLock<bool>>,
    pub addr: Arc<RwLock<SocketAddr>>,
    pub read_half: Arc<RwLock<OwnedReadHalf>>,
    pub write_half: Arc<RwLock<OwnedWriteHalf>>,
}

impl Client {
    pub async fn new(
        addr: SocketAddr,
        stream: TcpStream,
        player: Arc<Player>,
        protocol: Arc<Protocol>,
        logger: Arc<LogManager>,
    ) -> Arc<Self> {
        let (read, write) = stream.into_split();
        Arc::new(Self {
            player,
            logger,
            protocol,
            addr: Arc::new(RwLock::new(addr)),
            read_half: Arc::new(RwLock::new(read)),
            listening: Arc::new(RwLock::new(false)),
            write_half: Arc::new(RwLock::new(write)),
        })
    }

    // Main client loop to listen to the pooling of the incoming packets.
    // If no bytes are read the connection is closed.
    // Tries to parse bytes into a Packet struct. No penalty for invalid packets.
    // Calls Protocol to handle each valid packet in a tokio async task.
    pub async fn connect(self: Arc<Self>) {
        *self.listening.write().await = true;
        *self.player.connected.write().await = true;

        let mut buffer = [0; 4096];
        while *self.listening.read().await {
            let mut read_stream = self.read_half.write().await;
            let bytes_read = match read_stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => continue,
            };

            if let Ok(packet) = Packet::from_bytes(&buffer[..bytes_read]) {
                tokio::spawn({
                    let client = Arc::clone(&self);
                    let protocol = Arc::clone(&self.protocol);
                    async move {
                        protocol.handle_packet(client, packet).await;
                    }
                });
            }
        }

        self.disconnect().await;
    }

    pub async fn reconnect(self: Arc<Self>, stream: TcpStream, addr: SocketAddr) {
        let (read, write) = stream.into_split();

        *self.addr.write().await = addr;
        *self.read_half.write().await = read;
        *self.write_half.write().await = write;

        tokio::spawn({
            let client = self.clone();
            async move {
                client.connect().await;
            }
        });
    }

    pub async fn disconnect(self: Arc<Self>) {
        *self.listening.write().await = false;
        *self.player.connected.write().await = false;
    }

    pub async fn send_packet(self: Arc<Self>, packet: Packet) {
        let mut tries = 0;
        let bytes = packet.wrap();
        while tries < 30 {
            let mut write_guard = self.write_half.write().await;
            if let Err(_) = write_guard.write(&bytes).await {
                sleep(Duration::from_secs(2)).await;
                tries += 1;
                continue;
            }

            break;
        }
    }
}
