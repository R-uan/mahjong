use crate::{network::client_manager::ClientManager, utils::errors::Error};
use lolg::Lolg;
use std::{net::Ipv4Addr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};

pub struct Server {
    port: u16,
    pub logger: Arc<Lolg>,
    pub socket: Arc<TcpListener>,
    pub running: Arc<RwLock<bool>>,
    pub client_manager: Arc<ClientManager>,
}

impl Server {
    pub async fn create_instance(port: u16) -> Result<Arc<Server>, Error> {
        let host = Ipv4Addr::new(127, 0, 0, 1);
        let listener = TcpListener::bind((host, port))
            .await
            .map_err(|_| Error::InitializationFailed(4))?;

        let lolg = Lolg::init(port + 1, true)
            .await
            .map_err(|_| Error::InitializationFailed(5))?;

        Arc::clone(&lolg).listen().await;
        let cm = ClientManager::new(Arc::clone(&lolg)).await;

        let server = Server {
            running: Arc::clone(&lolg.running),
            client_manager: Arc::new(cm),
            socket: Arc::new(listener),
            logger: lolg,
            port,
        };

        return Ok(Arc::new(server));
    }

    pub async fn start(self: Arc<Self>) {
        let log_msg = &format!("Server initialized on port {}", &self.port);
        self.logger.debug(&log_msg).await;
        while *self.running.read().await {
            match self.socket.accept().await {
                Err(_) => continue,
                Ok((stream, addr)) => {
                    self.logger.debug(&format!("New client {addr}")).await;
                    Arc::clone(&self.client_manager).accept(stream, addr).await;
                    continue;
                }
            }
        }
        self.logger.debug("Server was closed").await;
    }
}
