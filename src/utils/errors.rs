#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to authenticate client ({0})")]
    AuthenticationFailed(u16),

    #[error("Failed to join match")]
    MatchAlreadyFull,

    #[error("Failed do parse game action packet {{0}}")]
    GameActionParsingFailed(u16),
}
