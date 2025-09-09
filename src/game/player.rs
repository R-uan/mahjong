use std::sync::Arc;

use tokio::sync::RwLock;

pub struct Player {
    pub view: View,
    pub id: Arc<String>,
    pub alias: Arc<RwLock<String>>,
    pub connected: Arc<RwLock<bool>>,
}

#[derive(Default)]
pub struct View {
    pub hand_tiles: String,
    pub open_tiles: String,
    pub discarded_tiles: String,
}

impl Player {
    pub fn new() -> Player {
        Player {
            view: View::default(),
            connected: Arc::new(RwLock::new(false)),
            id: Arc::new("".to_string()),
            alias: Arc::new(RwLock::new("".to_string())),
        }
    }
}
