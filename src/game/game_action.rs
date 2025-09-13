use crate::game::{errors::GameError, tiles::Tile};

pub enum Action {
    DISCARD = 0,
    KAN = 1,
    PON = 2,
    CHI = 3,
}

impl Action {
    pub fn from(value: i32) -> Option<Action> {
        match value {
            0 => Some(Action::DISCARD),
            1 => Some(Action::KAN),
            2 => Some(Action::PON),
            3 => Some(Action::CHI),
            _ => None,
        }
    }
}

pub struct GameAction {
    pub target: Tile,
    pub action: Action,
}

impl GameAction {
    fn from_bytes(b: Box<[u8]>) -> Result<GameAction, GameError> {
        let action_bytes = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        todo!()
    }
}
