use std::sync::Arc;

use crate::game::{game_action::GameAction, player::{Player, Seat}};

pub struct GameState {
    pub player1: Arc<Player>,
    pub player2: Arc<Player>,
    pub player3: Arc<Player>,
    pub player4: Arc<Player>,
    pub tile_wall: String,
}

impl GameState {
    pub fn start_game() -> Self {
        Self {
            tile_wall: "".to_string(),
            player1: Arc::new(Player::new(Seat::East)),
            player2: Arc::new(Player::new(Seat::West)),
            player3: Arc::new(Player::new(Seat::North)),
            player4: Arc::new(Player::new(Seat::South)),
        }
    }
}

pub struct GameManager {
    match_id: String,
    state: GameState,
}

impl GameManager {
    pub fn new_manager() -> GameManager {
        Self {
            match_id: String::new(),
            state: GameState::start_game(),
        }
    }

    fn apply_actions(&self, action: GameAction) {
        todo!()
    }

    fn reasign_seats(&self) {
        todo!()
    }

    pub async fn get_free_seat(&self) -> Option<Arc<Player>> {
        if !*self.state.player1.connected.read().await {
            return Some(Arc::clone(&self.state.player1));
        } else if !*self.state.player2.connected.read().await {
            return Some(Arc::clone(&self.state.player2));
        } else if !*self.state.player3.connected.read().await {
            return Some(Arc::clone(&self.state.player3));
        } else if !*self.state.player4.connected.read().await {
            return Some(Arc::clone(&self.state.player4));
        } else {
            return None;
        }
    }

    pub async fn assign_player(&self, id: &str, username: &str) -> Option<Arc<Player>> {
        match self.get_free_seat().await {
            Some(seat) => {
                *seat.connected.write().await = true;
                *seat.id.write().await = id.to_string();
                *seat.alias.write().await = username.to_string();
                return Some(seat);
            }
            None => None,
        }
    }
}
