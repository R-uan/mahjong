use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    game::enums::{PlayerStatus, Seat, Tile},
    utils::{errors::Error, models::JoinRequest},
};

pub struct Player {
    pub id: i32,
    pub alias: Arc<RwLock<String>>,
    pub connected: Arc<RwLock<bool>>,

    pub seat: Arc<RwLock<Seat>>,
    pub hand: Arc<RwLock<Vec<Arc<Tile>>>>,
    pub discarded: Arc<RwLock<Vec<Arc<Tile>>>>,
    pub player_state: Arc<RwLock<PlayerStatus>>,
}

impl Player {
    pub fn new(seat: Seat, req: &JoinRequest, hand: Vec<Arc<Tile>>) -> Player {
        Player {
            id: req.id,
            seat: Arc::new(RwLock::new(seat)),
            connected: Arc::new(RwLock::new(false)),
            alias: Arc::new(RwLock::new(req.alias.to_string())),
            player_state: Arc::new(RwLock::new(PlayerStatus::WAITING)),
            hand: Arc::new(RwLock::new(hand)),
            discarded: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn discard_tile(&self, target: &Tile) -> bool {
        let mut hand = self.hand.write().await;
        if let Some(pos) = hand
            .iter()
            .position(|t| t.kind == target.kind && t.copy == target.copy)
        {
            let tile = hand.remove(pos);
            self.discarded.write().await.push(tile);
            return true;
        }
        return false;
    }

    pub async fn get_hand(&self) -> Vec<i8> {
        let hand = self.hand.read().await;
        let vec: Vec<i8> = hand.iter().map(|tile| tile.kind as i8).collect();
        vec
    }

    pub async fn get_initial_view(&self) -> Result<Vec<u8>, Error> {
        let view = InitialPlayerView::get(&self).await;
        serde_cbor::to_vec(&view).map_err(|_| Error::SerializationFailed(10))
    }

    pub async fn check_ready(&self) -> bool {
        return *self.player_state.read().await == PlayerStatus::READY;
    }

    pub async fn set_ready(&self) {
        *self.player_state.write().await = PlayerStatus::READY;
    }

    pub async fn set_waiting(&self) {
        *self.player_state.write().await = PlayerStatus::WAITING;
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
        let hand = p.hand.read().await.to_owned();
        let seat = p.seat.read().await.to_owned();
        InitialPlayerView {
            is_first: seat == Seat::East,
            seat,
            hand,
        }
    }
}
