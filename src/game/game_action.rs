pub enum Action {
    KAN,
    PON,
    CHI,
    DISCARD,
}

pub struct GameAction {
    pub action: Action,
    pub target: String,
}

impl GameAction {
    pub fn new(action: Action, target: String) -> Self {
        Self { action, target }
    }
}
