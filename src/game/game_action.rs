use crate::{
    game::enums::{Action, Tile},
    utils::errors::Error,
};

pub struct GameAction {
    pub action: Action,
    pub target: Option<Tile>,
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
