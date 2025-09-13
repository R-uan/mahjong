#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Failed authentication")]
    FailedAuthentication,
    #[error("Failed to join match")]
    MatchIsFull,
}
