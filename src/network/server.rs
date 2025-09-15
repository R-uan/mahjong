use crate::game::game_state::GameManager;
use crate::network::client::ClientManager;
use crate::protocol::protocol::Protocol;
use crate::utils::log_manager::{LogLevel, LogManager};
use std::io::Error;
use std::{net::Ipv4Addr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};

pub struct Server {
    port: u16,
    pub logger: Arc<LogManager>,
    pub socket: Arc<TcpListener>,
    pub running: Arc<RwLock<bool>>,
    pub game_manager: Arc<GameManager>,
    pub client_manager: Arc<ClientManager>,
}

impl Server {
    pub async fn create_instance(port: u16) -> Result<Server, Error> {
        let running = Arc::new(RwLock::new(false));
        let host = Ipv4Addr::new(127, 0, 0, 1);
        let listener = TcpListener::bind((host, port)).await?;

        let lm = LogManager::new_instance(port + 1, running.clone()).await?;
        let gm = Arc::new(GameManager::new_instance(Arc::clone(&lm)));
        let protocol = Protocol::new(Arc::clone(&gm), Arc::clone(&lm));
        let cm = ClientManager::new_manager(Arc::new(protocol), Arc::clone(&lm));

        let server = Server {
            port,
            logger: lm,
            game_manager: gm,
            socket: Arc::new(listener),
            client_manager: Arc::new(cm),
            running: Arc::new(RwLock::new(false)),
        };

        return Ok(server);
    }

    pub async fn start(self: Arc<Self>) {
        *self.running.write().await = true;

        println!("Server running on port {}", &self.port);
        self.logger
            .send(
                LogLevel::Debug,
                &format!("Server running on port {}", &self.port),
                "SERVER",
            )
            .await;

        while *self.running.read().await {
            match self.socket.accept().await {
                Ok((stream, addr)) => {
                    self.logger
                        .send(
                            LogLevel::Info,
                            &format!("New client connected {addr}"),
                            "SERVER",
                        )
                        .await;

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
