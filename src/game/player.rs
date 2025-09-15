use std::sync::Arc;

use tokio::sync::RwLock;

use crate::game::tiles::{Tile, TileType};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Seat {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
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
    pub open: String,
    pub hand: Arc<RwLock<Vec<Tile>>>,
    pub discarded: Arc<RwLock<Vec<Tile>>>,
}

impl Player {
    pub fn new(seat: Seat, id: &str, alias: &str) -> Player {
        Player {
            view: Arc::new(View::default()),
            seat: Arc::new(RwLock::new(seat)),
            connected: Arc::new(RwLock::new(false)),
            id: Arc::new(RwLock::new(id.to_string())),
            alias: Arc::new(RwLock::new(alias.to_string())),
        }
    }

    pub async fn discard_tile(&self, target: TileType) -> bool {
        let mut hand = self.view.hand.write().await;
        if let Some(pos) = hand.iter().position(|t| t.kind == target) {
            let tile = hand.remove(pos);
            self.view.discarded.write().await.push(tile);
            return true;
        }
        return false;
    }
}
