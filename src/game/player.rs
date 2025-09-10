use std::sync::Arc;

use tokio::sync::RwLock;

pub enum Seat {
    North,
    South,
    East,
    West,
}

pub struct Player {
    pub view: Arc<View>,
    pub id: Arc<RwLock<String>>,
    pub seat: Arc<RwLock<Seat>>,
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
    pub fn new(seat: Seat) -> Player {
        Player {
            view: Arc::new(View::default()),
            seat: Arc::new(RwLock::new(seat)),
            connected: Arc::new(RwLock::new(false)),
            id: Arc::new(RwLock::new(String::new())),
            alias: Arc::new(RwLock::new(String::new())),
        }
    }
}
