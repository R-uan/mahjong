use crate::game::game_state::GameManager;
use crate::network::client::ClientManager;
use crate::network::protocol::Protocol;
use std::io::Error;
use std::{net::Ipv4Addr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};

pub struct Server {
    pub port: u16,
    pub socket: Arc<TcpListener>,
    pub running: Arc<RwLock<bool>>,
    pub game_manager: Arc<GameManager>,
    pub client_manager: Arc<ClientManager>,
}

impl Server {
    pub async fn create_instance(port: u16) -> Result<Server, Error> {
        let host = Ipv4Addr::new(127, 0, 0, 1);
        let listener = TcpListener::bind((host, port)).await?;
        let game_manager = Arc::new(GameManager::new_manager());
        let protocol = Protocol::new(Arc::clone(&game_manager));
        let client_manager = ClientManager::new_manager(Arc::new(protocol));

        let server = Server {
            port,
            game_manager,
            socket: Arc::new(listener),
            running: Arc::new(RwLock::new(false)),
            client_manager: Arc::new(client_manager),
        };

        return Ok(server);
    }

    pub async fn init(self: Arc<Self>) {
        while *self.running.read().await {
            match self.socket.accept().await {
                Ok(client) => {}
                Err(error) => {}
            }
        }
    }
}
