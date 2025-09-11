#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("0")]
    InvalidPacket,

    #[error("1")]
    MatchIsFull,
}
