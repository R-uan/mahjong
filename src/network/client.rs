use crate::game::player::Player;
use crate::network::protocol::Protocol;
use crate::protocol::packet::Packet;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;

pub struct ClientManager {
    pub protocol: Arc<Protocol>,
    pub authenticated_clients: Arc<RwLock<HashMap<String, Client>>>,
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
    pub async fn handle_client(self: Arc<Self>, mut stream: TcpStream, addr: SocketAddr) {
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

            let packet = Packet::from_bytes(&buffer[..read_bytes]);
            if let Some(player) = self.authenticate_client(&packet).await {
                let mut client_poll = self.authenticated_clients.write().await;
                let client = Client::new(stream, addr, player, Arc::clone(&self.protocol)).await;
                client_poll.insert(client.uuid.clone(), client);
                return;
            } else {
                attempts += 1;
            }
        }
    }

    pub async fn authenticate_client(&self, packet: &Packet) -> Option<Arc<Player>> {
        todo!()
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
    pub read_half: OwnedReadHalf,
    pub write_half: OwnedWriteHalf,
}

impl Client {
    pub async fn new(
        addr: SocketAddr,
        stream: TcpStream,
        player: Arc<Player>,
        protocol: Arc<Protocol>,
    ) -> Self {
        let uuid = player.id.read().await.clone();
        let (read, write) = stream.into_split();
        Self {
            uuid,
            addr,
            player,
            protocol,
            read_half: read,
            write_half: write,
        }
    }
}
