#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to authenticate client ({0})")]
    AuthenticationFailed(u32),

    #[error("Failed to join match")]
    MatchAlreadyFull,
}
