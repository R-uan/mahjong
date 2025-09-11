#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("Could not parse a valid GameAction")]
    GameActionParsing,
    #[error("Match is full")]
    NoAvailableSeats,
}
