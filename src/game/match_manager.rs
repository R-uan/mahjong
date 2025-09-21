use crate::{
    game::{
        game_action::GameAction,
        game_state::GameState,
        player::{Player, Seat},
        tiles::Tile,
    },
    utils::{errors::Error, log_manager::LogManager},
};
use std::sync::Arc;
use tokio::sync::{RwLock, watch};

#[derive(Clone, PartialEq, Eq)]
pub enum MatchStatus {
    Waiting = 0,
    Ready = 1,
    Finished = 2,
    Ongoing = 3,
    Interrupted = 4,
}

impl MatchStatus {
    pub fn wrap(&self) -> [u8; 4] {
        match &self {
            Self::Ready => [0x01, 0x00, 0x00, 0x00],
            Self::Waiting => [0x00, 0x00, 0x00, 0x00],
            Self::Ongoing => [0x03, 0x00, 0x00, 0x00],
            Self::Finished => [0x02, 0x00, 0x00, 0x00],
            Self::Interrupted => [0x04, 0x00, 0x00, 0x00],
        }
    }
}

pub struct MatchManager {
    match_id: String,
    state: GameState,
    logger: Arc<LogManager>,
    current_turn: Arc<RwLock<Seat>>,
    pub status: Arc<RwLock<MatchStatus>>,
    sttx: Arc<watch::Sender<MatchStatus>>,
}

impl MatchManager {
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
        if *self.current_turn.read().await != *player.seat.read().await {
            return Err(Error::DrawFailed(163));
        }

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
            return Err(Error::DiscardFailed(165));
        }

        let tile = action.target.ok_or(Error::TileParsingFailed)?;
        match player.discard_tile(&tile).await {
            true => return Ok(true),
            false => return Err(Error::DiscardFailed(164)),
        };
    }
}

impl MatchManager {
    pub fn new(log_manager: Arc<LogManager>, sender: watch::Sender<MatchStatus>) -> MatchManager {
        Self {
            logger: log_manager,
            match_id: String::new(),
            sttx: Arc::new(sender),
            state: GameState::start_game(),
            current_turn: Arc::new(RwLock::new(Seat::East)),
            status: Arc::new(RwLock::new(MatchStatus::Waiting)),
        }
    }

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

        self.change_status(MatchStatus::Ready).await;
        return Ok(true);
    }

    async fn change_status(&self, status: MatchStatus) {
        let mut status_guard = self.status.write().await;
        let _ = self.sttx.send(status.to_owned()).unwrap();
        *status_guard = status;
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
