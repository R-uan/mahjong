use std::fmt::Display;

use crate::{game::tiles::Tile, utils::errors::Error};

#[derive(PartialEq, Eq)]
pub enum Action {
    DRAW = 0,
    DISCARD = 1,
    KAN = 2,
    PON = 3,
    CHI = 4,
    RON = 5,
    TSUMO = 6,
}

impl Action {
    pub fn bytes(&self) -> [u8; 4] {
        let leading = match self {
            Self::DRAW => 0x00,
            Self::DISCARD => 0x01,
            Self::KAN => 0x02,
            Self::PON => 0x03,
            Self::CHI => 0x04,
            Self::RON => 0x05,
            Self::TSUMO => 0x06,
        };

        return [leading, 0x00, 0x00, 0x00];
    }

    pub fn get(value: u8) -> Option<Action> {
        match value {
            0 => Some(Action::DRAW),
            1 => Some(Action::DISCARD),
            2 => Some(Action::KAN),
            3 => Some(Action::PON),
            4 => Some(Action::CHI),
            5 => Some(Action::RON),
            6 => Some(Action::TSUMO),
            _ => None,
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DRAW => write!(f, "DRAW"),
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
