use serde::Serialize;
use std::io::Error;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

pub struct LogManager {
    pub socket: TcpListener,
    pub running: Arc<RwLock<bool>>,
    pub clients: Arc<RwLock<Vec<LoggerClient>>>,
}

pub struct LoggerClient {
    pub stream: Arc<RwLock<TcpStream>>,
}

impl LogManager {
    pub async fn new(port: u16, running: Arc<RwLock<bool>>) -> Result<Arc<Self>, Error> {
        let host = Ipv4Addr::new(127, 0, 0, 1);

        let listener = TcpListener::bind((host, port)).await?;
        let clients = Arc::new(RwLock::new(Vec::new()));

        println!("Log manager on port {port}");

        let lm = Arc::new(Self {
            clients,
            running,
            socket: listener,
        });

        tokio::spawn({
            let lm_clone = Arc::clone(&lm);
            async move {
                while *lm_clone.running.read().await {
                    match lm_clone.socket.accept().await {
                        Err(_) => {}
                        Ok((stream, _)) => {
                            let client = LoggerClient {
                                stream: Arc::new(RwLock::new(stream)),
                            };
                            lm_clone.clients.write().await.push(client);
                        }
                    }
                }
            }
        });

        Ok(Arc::clone(&lm))
    }

    pub async fn debug(&self, m: String, t: &str) {
        self.send(LogLevel::Debug, &m, t).await;
    }

    pub async fn info(&self, m: String, t: &str) {
        self.send(LogLevel::Info, &m, t).await;
    }

    pub async fn error(&self, m: String, t: &str) {
        self.send(LogLevel::Error, &m, t).await;
    }

    pub async fn warning(&self, m: String, t: &str) {
        self.send(LogLevel::Warning, &m, t).await;
    }

    async fn send(&self, l: LogLevel, m: &str, t: &str) {
        let entry = LogEntry::new(l, m, t);
        let mut cleanup: Vec<usize> = Vec::new();
        if let Ok(encoded) = serde_cbor::to_vec(&entry) {
            let bytes = encoded.into_boxed_slice();
            let clients = self.clients.read().await;

            for i in 0..clients.len() {
                let client = &clients[i];
                if let Err(_) = client.stream.write().await.write(&bytes).await {
                    cleanup.push(i);
                }
            }
        }

        for index in cleanup {
            let mut clients = self.clients.write().await;
            clients.remove(index);
        }
    }
}

#[derive(Serialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub target: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: &str, target: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            level,
            timestamp,
            target: target.to_owned(),
            message: message.to_owned(),
        }
    }
}

#[derive(Serialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}
