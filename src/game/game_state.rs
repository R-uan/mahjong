use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    game::{
        errors::GameError,
        game_action::{Action, GameAction},
        player::{Player, Seat},
        tiles::{Tile, TileType},
    },
    utils::{
        errors::Error,
        log_manager::{LogLevel, LogManager},
    },
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

    pub async fn start_game(&self, player: Arc<Player>) {}

    pub async fn next_player(&self) {
        *self.current_seat.write().await = match *self.current_seat.read().await {
            Seat::East => Seat::North,
            Seat::North => Seat::West,
            Seat::West => Seat::South,
            Seat::South => Seat::East,
        };
    }

    pub async fn apply_actions(&self, p: Arc<Player>, a: GameAction) -> Result<bool, Error> {
        let result = match a.action {
            Action::DISCARD => {
                let target = a.target.ok_or(Error::TileParsingFailed)?;
                return Ok(p.discard_tile(&target).await);
            }
            _ => Err(Error::GameActionFailed),
        };

        self.logger
            .send(
                LogLevel::Info,
                &format!(
                    "{} {} {}",
                    &p.alias.read().await,
                    &a.action,
                    if let Some(tile) = &a.target {
                        &tile.kind.to_string()
                    } else {
                        ""
                    }
                ),
                "Game Manager",
            )
            .await;

        return result;
    }

    // Player draw a tile
    //
    // Ok
    // - Arc reference of the drawn tile
    // Err
    // - 151 : Unable to draw tile (Hand full)
    // - 152 : Unable to draw tile (Wall Empty)
    async fn draw(&self, player: Arc<Player>) -> Result<Arc<Tile>, Error> {
        let mut hand = player.view.hand.write().await;
        if hand.len() >= 14 {
            return Err(Error::DrawFailed(151));
        }

        let mut wall = self.state.wall.write().await;
        if wall.len() == 0 {
            return Err(Error::DrawFailed(152));
        }

        let tile = wall.pop().ok_or(Error::DrawFailed(152))?;
        let tile_clone = Arc::clone(&tile);

        hand.push(tile);
        return Ok(tile_clone);
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
