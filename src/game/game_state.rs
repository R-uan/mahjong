use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    game::{
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
    running: Arc<RwLock<bool>>,
    current_turn: Arc<RwLock<Seat>>,
}

impl GameManager {
    pub fn new_instance(lm: Arc<LogManager>) -> GameManager {
        Self {
            logger: lm,
            match_id: String::new(),
            state: GameState::start_game(),
            running: Arc::new(RwLock::new(false)),
            current_turn: Arc::new(RwLock::new(Seat::East)),
        }
    }

    pub async fn next_player(&self) -> Result<Arc<Player>, Error> {
        let mut guard = self.current_turn.write().await;
        let next_seat = match *guard {
            Seat::East => Seat::North,
            Seat::North => Seat::West,
            Seat::West => Seat::South,
            Seat::South => Seat::East,
        };

        *guard = next_seat;
        if let Some(player) = self.state.player_pool.read().await.get(&next_seat) {
            return Ok(Arc::clone(player));
        }
        return Err(Error::NextPlayerFailed);
    }

    pub async fn draw(&self, player: Arc<Player>) -> Result<Arc<Tile>, Error> {
        let mut hand = player.view.hand.write().await;
        if hand.len() >= 14 {
            return Err(Error::DrawFailed(161));
        }

        let mut wall = self.state.wall.write().await;
        if wall.len() == 0 {
            return Err(Error::DrawFailed(162));
        }

        let tile = wall.pop().ok_or(Error::DrawFailed(162))?;
        let tile_clone = Arc::clone(&tile);

        hand.push(tile);
        return Ok(tile_clone);
    }

    pub async fn discard(&self, player: Arc<Player>, action: GameAction) -> Result<bool, Error> {
        if *self.current_turn.read().await != *player.seat.read().await {
            return Ok(false);
        }

        let tile = action.target.ok_or(Error::TileParsingFailed)?;
        match player.discard_tile(&tile).await {
            true => return Ok(true),
            false => return Err(Error::DiscardFailed(163)),
        };
    }
}

impl GameManager {
    pub async fn check_ready(&self) -> Result<bool, Error> {
        let player_pool = self.state.player_pool.read().await;
        // Check if there are four players connected
        if player_pool.len() == 4 {
            return Err(Error::MatchStartFailed(151));
        }

        let _ = player_pool
            .get(&Seat::East)
            .ok_or(Error::MatchStartFailed(152))?;
        let _ = player_pool
            .get(&Seat::West)
            .ok_or(Error::MatchStartFailed(153))?;
        let _ = player_pool
            .get(&Seat::North)
            .ok_or(Error::MatchStartFailed(154))?;
        let _ = player_pool
            .get(&Seat::South)
            .ok_or(Error::MatchStartFailed(155))?;

        return Ok(true);
    }

    pub async fn wrap(&self) {
        todo!()
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

    pub async fn assign_player(&self, id: &str, alias: &str) -> Result<Arc<Player>, Error> {
        match self.get_free_seat().await {
            None => Err(Error::NoAvailableSeats),
            Some(seat) => {
                let player = Arc::new(Player::new(seat.clone(), id, alias));
                let mut player_pool_guard = self.state.player_pool.write().await;
                player_pool_guard.insert(seat, player.clone());
                return Ok(player);
            }
        }
    }
}
