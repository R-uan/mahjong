#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Server Related Errors
    #[error("network error: failed to authenticate client ({0})")]
    AuthenticationFailed(u16),

    // Client Related Errors
    #[error("CLIENT ERROR ({0})")]
    ReconnectionFailed(u16),

    #[error("CLIENT ERROR (56) ")]
    ConnectionNeeded,

    // Protocol Related Errors
    #[error("Could not parse received packet ({0})")]
    PacketParsingFailed(u16),

    #[error("request error: failed to parse target Tile")]
    TileParsingFailed,

    #[error("request error: failed do parse game action packet ({0})")]
    GameActionParsingFailed(u16),

    // Game Relasted Errors
    #[error("request error: failed to join match")]
    MatchAlreadyFull,

    #[error("game error: failed to apply game action")]
    GameActionFailed,

    #[error("game error: unable to draw a tile ({0})")]
    DrawFailed(u16),

    #[error("game error: could not start match ({0})")]
    MatchStartFailed(u16),

    #[error("game error: could not get next player")]
    NextPlayerFailed,

    #[error("")]
    NoAvailableSeats,

    #[error("game error: could not discard tile ({0})")]
    DiscardFailed(u16),
}
