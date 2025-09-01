use crate::game::player::Player;
use crate::network::protocol::Protocol;
use crate::protocol::packet::{Packet, PacketType};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;

pub struct ClientManager {
    pub protocol: Arc<Protocol>,
    pub authenticated_clients: Arc<RwLock<HashMap<String, Arc<Client>>>>,
}

impl ClientManager {
    pub fn new_manager(protocol: Arc<Protocol>) -> Self {
        Self {
            protocol,
            authenticated_clients: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    // Handles the initial client state (unauthenticated) and awaits for authentication
    // Create a Client struct when successfully authenticated
    pub async fn accept(&self, mut stream: TcpStream, addr: SocketAddr) {
        let mut buffer = [0; 1024];
        let mut attempts = 0;
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
                if packet.packet_type == PacketType::Authentication {
                    if let Some(player) = self.authenticate_client(&packet).await {
                        let mut client_poll = self.authenticated_clients.write().await;
                        let client = Client::new(addr, stream, player, self.protocol.clone()).await;
                        tokio::spawn({
                            let client_clone = Arc::clone(&client);
                            async move {
                                client_clone.listen().await;
                            }
                        });
                        client_poll.insert(client.uuid.clone(), client);
                        return;
                    }
                }
            }

            attempts += 1;
        }
    }

    // Calls into the player auth server to authenticate player.
    pub async fn authenticate_client(&self, _: &Packet) -> Option<Arc<Player>> {
        let player = self
            .protocol
            .assign_player("placeholder", "placeholder")
            .await;
        return player;
    }
}

pub struct Client {
    pub uuid: String,
    pub addr: SocketAddr,
    pub player: Arc<Player>,
    pub protocol: Arc<Protocol>,
    pub read_half: Arc<RwLock<OwnedReadHalf>>,
    pub write_half: Arc<RwLock<OwnedWriteHalf>>,
}

impl Client {
    pub async fn new(
        addr: SocketAddr,
        stream: TcpStream,
        player: Arc<Player>,
        protocol: Arc<Protocol>,
    ) -> Arc<Self> {
        let uuid = player.id.read().await.clone();
        let (read, write) = stream.into_split();
        Arc::new(Self {
            uuid,
            addr,
            player,
            protocol,
            read_half: Arc::new(RwLock::new(read)),
            write_half: Arc::new(RwLock::new(write)),
        })
    }

    pub async fn listen(self: Arc<Self>) {
        let mut buffer = [0; 4096];
        let mut read_stream = self.read_half.write().await;
        loop {
            let bytes_read = match read_stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
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
    }
}
