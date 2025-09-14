use crate::game::game_state::GameManager;
use crate::network::client::ClientManager;
use crate::protocol::protocol::Protocol;
use crate::utils::log_manager::LogManager;
use std::io::Error;
use std::{net::Ipv4Addr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};

pub struct Server {
    pub socket: Arc<TcpListener>,
    pub running: Arc<RwLock<bool>>,
    pub game_manager: Arc<GameManager>,
    pub client_manager: Arc<ClientManager>,
}

impl Server {
    pub async fn create_instance(port: u16) -> Result<Server, Error> {
        let running = Arc::new(RwLock::new(false));
        let host = Ipv4Addr::new(127, 0, 0, 1);
        let lm = Arc::new(LogManager::new_instance(port + 1, running.clone()).await?);
        let listener = TcpListener::bind((host, port)).await?;
        let game_manager = Arc::new(GameManager::new_instance(Arc::clone(&lm)));
        let protocol = Protocol::new(Arc::clone(&game_manager), Arc::clone(&lm));
        let client_manager = ClientManager::new_manager(Arc::new(protocol));

        let server = Server {
            game_manager,
            socket: Arc::new(listener),
            running: Arc::new(RwLock::new(false)),
            client_manager: Arc::new(client_manager),
        };

        return Ok(server);
    }

    pub async fn start(self: Arc<Self>) {
        while *self.running.read().await {
            match self.socket.accept().await {
                Ok((stream, addr)) => {
                    tokio::spawn({
                        let server = self.clone();
                        async move {
                            server.client_manager.accept(stream, addr).await;
                        }
                    });
                    continue;
                }
                Err(_) => {}
            }
        }
    }
}
