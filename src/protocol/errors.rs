#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("0")]
    InvalidPacket,
}
