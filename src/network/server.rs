use crate::network::client_manager::ClientManager;
use crate::utils::log_manager::LogManager;
use std::io::Error;
use std::{net::Ipv4Addr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};

pub struct Server {
    port: u16,
    pub logger: Arc<LogManager>,
    pub socket: Arc<TcpListener>,
    pub running: Arc<RwLock<bool>>,
    pub client_manager: Arc<ClientManager>,
}

impl Server {
    pub async fn create_instance(port: u16) -> Result<Server, Error> {
        let running = Arc::new(RwLock::new(false));
        let host = Ipv4Addr::new(127, 0, 0, 1);
        let listener = TcpListener::bind((host, port)).await?;

        let lm = LogManager::new(port + 1, running.clone()).await?;
        let cm = ClientManager::new(Arc::clone(&lm)).await;

        let server = Server {
            port,
            running,
            logger: lm,
            socket: Arc::new(listener),
            client_manager: Arc::new(cm),
        };

        return Ok(server);
    }

    pub async fn start(self: Arc<Self>) {
        *self.running.write().await = true;

        println!("Server running on port {}", &self.port);
        self.logger
            .debug(format!("Server running on port {}", &self.port), "SERVER")
            .await;

        while *self.running.read().await {
            match self.socket.accept().await {
                Err(_) => continue,
                Ok((stream, addr)) => {
                    self.logger
                        .info(format!("New client connected {addr}"), "SERVER")
                        .await;
                    Arc::clone(&self.client_manager).accept(stream, addr).await;
                    continue;
                }
            }
        }
    }
}
