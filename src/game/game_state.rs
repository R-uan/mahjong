use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::
    game::{
        player::{Player, Seat},
        tiles::{Tile, TileType},
    };

pub struct GameState {
    pub turn: Arc<RwLock<i32>>,
    pub wall: Arc<RwLock<Vec<Arc<Tile>>>>,
    pub last_discard: Arc<RwLock<Option<TileType>>>,
    pub player_pool: Arc<RwLock<HashMap<Seat, Arc<Player>>>>,
}

impl GameState {
    pub fn start_game() -> Self {
        Self {
            turn: Arc::new(RwLock::new(0)),
            wall: Arc::new(RwLock::new(Vec::new())),
            last_discard: Arc::new(RwLock::new(None)),
            player_pool: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}