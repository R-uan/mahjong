use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    game::tiles::Tile,
    utils::{errors::Error, models::JoinRequest},
};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Seat {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

#[derive(PartialEq, Eq)]
pub enum PlayerState {
    WAITING,
    DRAW,
    DISCARD,
    READY,
}

pub struct Player {
    pub id: i32,
    pub view: Arc<View>,
    pub seat: Arc<RwLock<Seat>>,
    pub alias: Arc<RwLock<String>>,
    pub connected: Arc<RwLock<bool>>,
    pub player_state: Arc<RwLock<PlayerState>>,
}

#[derive(Default)]
pub struct View {
    pub open: String,
    pub hand: Arc<RwLock<Vec<Arc<Tile>>>>,
    pub discarded: Arc<RwLock<Vec<Arc<Tile>>>>,
}

impl View {
    pub fn new(hand: Vec<Arc<Tile>>) -> Self {
        View {
            open: String::new(),
            hand: Arc::new(RwLock::new(hand)),
            discarded: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Player {
    pub fn new(seat: Seat, req: &JoinRequest, hand: Vec<Arc<Tile>>) -> Player {
        Player {
            id: req.id,
            view: Arc::new(View::new(hand)),
            seat: Arc::new(RwLock::new(seat)),
            connected: Arc::new(RwLock::new(false)),
            alias: Arc::new(RwLock::new(req.alias.to_string())),
            player_state: Arc::new(RwLock::new(PlayerState::WAITING)),
        }
    }

    pub async fn discard_tile(&self, target: &Tile) -> bool {
        let mut hand = self.view.hand.write().await;
        if let Some(pos) = hand
            .iter()
            .position(|t| t.kind == target.kind && t.copy == target.copy)
        {
            let tile = hand.remove(pos);
            self.view.discarded.write().await.push(tile);
            return true;
        }
        return false;
    }

    pub async fn get_initial_view(&self) -> Result<Vec<u8>, Error> {
        let view = InitialPlayerView::get(&self).await;
        serde_cbor::to_vec(&view).map_err(|_| Error::SerializationFailed(10))
    }

    pub async fn check_ready(&self) -> bool {
        return *self.player_state.read().await == PlayerState::READY;
    }

    pub async fn set_ready(&self) {
        *self.player_state.write().await = PlayerState::READY;
    }

    pub async fn set_waiting(&self) {
        *self.player_state.write().await = PlayerState::WAITING;
    }
}

#[derive(Deserialize, Serialize)]
pub struct InitialPlayerView {
    pub seat: Seat,
    pub is_first: bool,
    pub hand: Vec<Arc<Tile>>,
}

impl InitialPlayerView {
    pub async fn get(p: &Player) -> Self {
        let hand = p.view.hand.read().await.to_owned();
        let seat = p.seat.read().await.to_owned();
        InitialPlayerView {
            is_first: seat == Seat::East,
            seat,
            hand,
        }
    }
}
