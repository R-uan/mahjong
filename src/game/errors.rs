#[derive(Debug, thiserror::Error)]
pub enum GameErrors {
    #[error("Could not parse a valid GameAction")]
    GameActionParsing,
}
