use crate::{
    game::{
        game_action::GameAction,
        game_state::GameState,
        player::{Player, Seat},
        tiles::Tile,
    },
    utils::{errors::Error, models::JoinRequest},
};
use lolg::Lolg;
use std::{fmt::Display, sync::Arc};
use tokio::sync::{RwLock, watch};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MatchStatus {
    Waiting = 0,
    Ongoing = 1,
    Finished = 2,
    Interrupted = 3,
}

impl Display for MatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Waiting => "Waiting",
            Self::Finished => "Finished",
            Self::Ongoing => "Ongoing",
            Self::Interrupted => "Interrupted",
        };

        write!(f, "{}", s)
    }
}

impl MatchStatus {
    pub fn bytes(&self) -> [u8; 4] {
        match &self {
            Self::Waiting => [0x00, 0x00, 0x00, 0x00],
            Self::Ongoing => [0x01, 0x00, 0x00, 0x00],
            Self::Finished => [0x02, 0x00, 0x00, 0x00],
            Self::Interrupted => [0x03, 0x00, 0x00, 0x00],
        }
    }
}

// RULES FOR THIS MANAGER
// - IT SHOULD NOT HAVE TO CREATE ANY PACKETS AS IT HAS NO DIRECT ACCESS TO PROTOCOL
//
pub struct MatchManager {
    match_id: String,
    logger: Arc<Lolg>,
    pub state: Arc<GameState>,
    current_turn: Arc<RwLock<Seat>>,
    pub status: Arc<RwLock<MatchStatus>>,
    sttx: Arc<watch::Sender<MatchStatus>>,
}

impl MatchManager {
    pub async fn next_turn(&self) -> Result<Arc<Player>, Error> {
        let mut guard = self.current_turn.write().await;
        let next_seat = match *guard {
            Seat::East => Seat::North,
            Seat::North => Seat::West,
            Seat::West => Seat::South,
            Seat::South => Seat::East,
        };

        *guard = next_seat;
        if let Some(player) = self.state.player_pool.read().await.get(&next_seat) {
            let mut turn_guard = self.state.turn.write().await;
            *turn_guard += 1;
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

    pub async fn discard(&self, player: Arc<Player>, action: GameAction) -> Result<Tile, Error> {
        if *self.current_turn.read().await != *player.seat.read().await {
            return Err(Error::DiscardFailed(165));
        }

        let tile = action.target.ok_or(Error::TileParsingFailed)?;
        match player.discard_tile(&tile).await {
            true => return Ok(tile),
            false => return Err(Error::DiscardFailed(164)),
        };
    }
}

impl MatchManager {
    pub fn new(log_manager: Arc<Lolg>, sender: watch::Sender<MatchStatus>) -> MatchManager {
        Self {
            logger: log_manager,
            match_id: String::new(),
            sttx: Arc::new(sender),
            state: Arc::new(GameState::start_game()),
            current_turn: Arc::new(RwLock::new(Seat::East)),
            status: Arc::new(RwLock::new(MatchStatus::Waiting)),
        }
    }

    async fn check_seats(&self) -> Result<(), Error> {
        let player_pool = self.state.player_pool.read().await;
        if player_pool.len() != 4 {
            return Err(Error::MatchStartFailed(151));
        }

        player_pool
            .get(&Seat::East)
            .ok_or(Error::MatchStartFailed(152))?
            .check_ready()
            .await
            .then(|| 0)
            .ok_or(Error::MatchStartFailed(152))?;

        player_pool
            .get(&Seat::West)
            .ok_or(Error::MatchStartFailed(153))?
            .check_ready()
            .await
            .then(|| 0)
            .ok_or(Error::MatchStartFailed(153))?;

        player_pool
            .get(&Seat::North)
            .ok_or(Error::MatchStartFailed(154))?
            .check_ready()
            .await
            .then(|| 0)
            .ok_or(Error::MatchStartFailed(154))?;

        player_pool
            .get(&Seat::South)
            .ok_or(Error::MatchStartFailed(155))?
            .check_ready()
            .await
            .then(|| 0)
            .ok_or(Error::MatchStartFailed(155))?;

        Ok(())
    }

    pub async fn check_ready(&self) -> bool {
        let Err(error) = self.check_seats().await else {
            self.logger
                .debug(&format!("Match is ready for initialization."))
                .await;
            return true;
        };
        self.logger
            .debug(&format!("Match is not ready yet: {error}."))
            .await;
        return false;
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

    pub async fn get_initial_hand(&self) -> Vec<Arc<Tile>> {
        let mut wall_guard = self.state.wall.write().await;
        let len = wall_guard.len();
        let drain_start = len.saturating_sub(13);
        wall_guard.drain(drain_start..).collect()
    }

    pub async fn assign_player(&self, req: &JoinRequest) -> Result<Arc<Player>, Error> {
        match self.get_free_seat().await {
            None => Err(Error::NoAvailableSeats),
            Some(seat) => {
                let hand = self.get_initial_hand().await;
                let player = Arc::new(Player::new(seat.clone(), &req, hand));
                let mut player_pool_guard = self.state.player_pool.write().await;
                player_pool_guard.insert(seat, player.clone());
                return Ok(player);
            }
        }
    }
}
