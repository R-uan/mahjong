use std::fmt::Display;

use crate::{game::tiles::Tile, utils::errors::Error};

#[derive(PartialEq, Eq)]
pub enum Action {
    DISCARD = 0,
    KAN = 1,
    PON = 2,
    CHI = 3,
    RON = 4,
    TSUMO = 5,
}

impl Action {
    pub fn get(value: u8) -> Option<Action> {
        match value {
            0 => Some(Action::DISCARD),
            1 => Some(Action::KAN),
            2 => Some(Action::PON),
            3 => Some(Action::CHI),
            4 => Some(Action::RON),
            5 => Some(Action::TSUMO),
            _ => None,
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DISCARD => write!(f, "DISCARD"),
            Self::KAN => write!(f, "KAN"),
            Self::PON => write!(f, "PON"),
            Self::CHI => write!(f, "CHI"),
            Self::RON => write!(f, "RON"),
            Self::TSUMO => write!(f, "TSUMO"),
        }
    }
}

pub struct GameAction {
    pub action: Action,
    pub target: Option<Tile>,
    // I removed the target property cause the epiphany,
    // that the target will always be the last discarded
}

impl GameAction {
    pub fn parse(b: &Box<[u8]>) -> Result<GameAction, Error> {
        match Action::get(b[0]) {
            None => return Err(Error::GameActionParsingFailed(1)),
            Some(action) => {
                return Ok(GameAction {
                    target: match action {
                        Action::DISCARD => Some(Tile::from_bytes(b[1], b[2])?),
                        _ => None,
                    },
                    action,
                });
            }
        }
    }
}
