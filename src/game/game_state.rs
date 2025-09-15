use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    game::{
        errors::GameError,
        game_action::GameAction,
        player::{Player, Seat},
        tiles::TileType,
    },
    utils::log_manager::LogManager,
};

pub struct GameState {
    pub tile_wall: String,
    pub turn: Arc<RwLock<i32>>,
    pub last_discard: Arc<RwLock<Option<TileType>>>,
    pub player_pool: Arc<RwLock<HashMap<Seat, Arc<Player>>>>,
}

impl GameState {
    pub fn start_game() -> Self {
        Self {
            tile_wall: "".to_string(),
            turn: Arc::new(RwLock::new(0)),
            last_discard: Arc::new(RwLock::new(None)),
            player_pool: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub struct GameManager {
    match_id: String,
    state: GameState,
    logger: Arc<LogManager>,
    current_seat: Arc<RwLock<Seat>>,
}

impl GameManager {
    pub fn new_instance(lm: Arc<LogManager>) -> GameManager {
        Self {
            logger: lm,
            match_id: String::new(),
            state: GameState::start_game(),
            current_seat: Arc::new(RwLock::new(Seat::East)),
        }
    }

    pub async fn next_player(&self) {
        *self.current_seat.write().await = match *self.current_seat.read().await {
            Seat::East => Seat::North,
            Seat::North => Seat::West,
            Seat::West => Seat::South,
            Seat::South => Seat::East,
        };
    }

    fn apply_actions(&self, action: GameAction) {
        todo!()
    }

    fn reasign_seats(&self) {
        todo!()
    }

    pub async fn get_free_seat(&self) -> Option<Seat> {
        let player_pool_guard = self.state.player_pool.read().await;
        return if player_pool_guard.get(&Seat::East).is_none() {
            Some(Seat::East)
        } else if player_pool_guard.get(&Seat::North).is_none() {
            Some(Seat::North)
        } else if player_pool_guard.get(&Seat::West).is_none() {
            Some(Seat::West)
        } else if player_pool_guard.get(&Seat::South).is_none() {
            Some(Seat::South)
        } else {
            None
        };
    }

    pub async fn assign_player(&self, id: &str, alias: &str) -> Result<Arc<Player>, GameError> {
        match self.get_free_seat().await {
            None => Err(GameError::NoAvailableSeats),
            Some(seat) => {
                let player = Arc::new(Player::new(seat.clone(), id, alias));
                let mut player_pool_guard = self.state.player_pool.write().await;
                player_pool_guard.insert(seat, player.clone());
                return Ok(player);
            }
        }
    }
}
