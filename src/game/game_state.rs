use std::sync::Arc;

use crate::game::player::Player;

pub struct GameState {
    pub south_seat: Arc<Player>,
    pub north_seat: Arc<Player>,
    pub east_seat: Arc<Player>,
    pub west_seat: Arc<Player>,
    pub tile_wall: String,
}

impl GameState {
    pub fn start_game() -> Self {
        Self {
            south_seat: Arc::new(Player::new()),
            north_seat: Arc::new(Player::new()),
            east_seat: Arc::new(Player::new()),
            west_seat: Arc::new(Player::new()),
            tile_wall: "".to_string(),
        }
    }
}

pub struct GameManager {
    state: GameState,
}

impl GameManager {
    pub fn new_manager() -> GameManager {
        Self {
            state: GameState::start_game(),
        }
    }

    pub async fn get_free_seat(&self) -> Option<Arc<Player>> {
        if !*self.state.south_seat.connected.read().await {
            return Some(Arc::clone(&self.state.south_seat));
        } else if !*self.state.north_seat.connected.read().await {
            return Some(Arc::clone(&self.state.north_seat));
        } else if !*self.state.east_seat.connected.read().await {
            return Some(Arc::clone(&self.state.east_seat));
        } else if !*self.state.west_seat.connected.read().await {
            return Some(Arc::clone(&self.state.west_seat));
        } else {
            return None;
        }
    }
}
