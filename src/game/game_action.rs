use crate::utils::errors::Error;

pub enum Action {
    DISCARD = 0,
    KAN = 1,
    PON = 2,
    CHI = 3,
}

impl Action {
    pub fn get(value: i32) -> Option<Action> {
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
    pub action: Action,
    // I removed the target property cause the epiphany,
    // that the target will always be the last discarded
}

impl GameAction {
    fn from_bytes(b: Box<[u8]>) -> Result<GameAction, Error> {
        match Action::get(u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as i32) {
            Some(action) => return Ok(GameAction { action }),
            None => return Err(Error::GameActionParsingFailed(1)),
        }
    }
}
