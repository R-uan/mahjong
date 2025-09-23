use crate::game::player::Player;
use crate::protocol::packet::{Packet, WriteBytesExt};
use crate::protocol::protocol::Protocol;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{RwLock, broadcast};
use tokio::time::sleep;

pub struct Client {
    pub id: String,
    pub player: Arc<Player>,
    pub protocol: Arc<Protocol>,
    pub listening: Arc<RwLock<bool>>,
    pub addr: Arc<RwLock<SocketAddr>>,
    pub read_half: Arc<RwLock<OwnedReadHalf>>,
    pub write_half: Arc<RwLock<OwnedWriteHalf>>,
    pub bcrx: Arc<RwLock<broadcast::Receiver<Packet>>>,
}

impl Client {
    pub async fn new(
        id: String,
        addr: SocketAddr,
        stream: TcpStream,
        player: Arc<Player>,
        protocol: Arc<Protocol>,
        bcrx: broadcast::Receiver<Packet>,
    ) -> Arc<Self> {
        let (read, write) = stream.into_split();
        Arc::new(Self {
            id,
            player,
            protocol,
            addr: Arc::new(RwLock::new(addr)),
            bcrx: Arc::new(RwLock::new(bcrx)),
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
        tokio::spawn({
            let self_clone = Arc::clone(&self);
            async move {
                let mut bcrx = self_clone.bcrx.write().await;
                while *self_clone.listening.read().await {
                    if let Ok(global_packet) = bcrx.recv().await {
                        self_clone.send_packet(&global_packet).await;
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut buffer = [0; 4096];
            *self.listening.write().await = true;
            *self.player.connected.write().await = true;

            while *self.listening.read().await {
                let mut read_stream = self.read_half.write().await;
                let bytes_read = match read_stream.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => continue,
                };

                match Packet::from_bytes(&buffer[..bytes_read]) {
                    Err(error) => self.send_packet(&Packet::error(0, error)).await,
                    Ok(packet) => {
                        Arc::clone(&self.protocol)
                            .handle_packet(Arc::clone(&self), packet)
                            .await
                    }
                };
            }

            self.disconnect().await;
        });
    }

    pub async fn reconnect(self: Arc<Self>, stream: TcpStream, addr: SocketAddr) {
        let (read, write) = stream.into_split();
        *self.addr.write().await = addr;
        *self.read_half.write().await = read;
        *self.write_half.write().await = write;
        Arc::clone(&self).connect().await;
    }

    pub async fn disconnect(self: Arc<Self>) {
        *self.listening.write().await = false;
        *self.player.connected.write().await = false;
    }

    pub async fn send_packet(&self, packet: &Packet) {
        let mut tries = 0;
        while tries < 30 {
            let mut write_guard = self.write_half.write().await;
            if let Err(_) = write_guard.send_packet(packet).await {
                sleep(Duration::from_secs(2)).await;
                tries += 1;
                continue;
            }

            break;
        }
    }
}
