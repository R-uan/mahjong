use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, tcp::OwnedWriteHalf},
};

use crate::utils::errors::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum PacketKind {
    Setup = 1,
    Action = 2,
    Broadcast = 3,
    Error = 255,
}

impl PacketKind {
    pub fn from_byte(byte: u32) -> Option<Self> {
        match byte as i32 {
            1 => Some(Self::Setup),
            2 => Some(Self::Action),
            3 => Some(Self::Broadcast),
            255 => Some(Self::Error),
            _ => None,
        }
    }

    pub fn bytes(&self) -> [u8; 4] {
        match self {
            PacketKind::Setup => [0x01, 0x00, 0x00, 0x00],
            PacketKind::Action => [0x02, 0x00, 0x00, 0x00],
            PacketKind::Error => [0x05, 0x00, 0x00, 0x00],
            PacketKind::Broadcast => [0x03, 0x00, 0x00, 0x00],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub id: i32,
    pub size: i32,
    pub body: Box<[u8]>,
    pub kind: PacketKind,
}

impl Packet {
    pub fn from_bytes(b: &[u8]) -> Result<Packet, Error> {
        if b.len() < 10 {
            return Err(Error::PacketParsingFailed(101));
        }
        match PacketKind::from_byte(u32::from_le_bytes([b[4], b[5], b[6], b[7]])) {
            None => return Err(Error::PacketParsingFailed(102)),
            Some(kind) => {
                let id = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                let size = u32::from_le_bytes([b[8], b[9], b[10], b[11]]);
                let body = b[12..b.len() - 2].to_vec().into_boxed_slice();
                return Ok(Self {
                    id: id as i32,
                    size: size as i32,
                    body,
                    kind,
                });
            }
        }
    }

    pub fn create(id: i32, packet_kind: PacketKind, body: &[u8]) -> Packet {
        Self {
            id,
            kind: packet_kind,
            size: (8 + 2 + body.len()) as i32,
            body: body.to_vec().into_boxed_slice(),
        }
    }

    pub fn error(id: i32, error: Error) -> Packet {
        return Packet::create(id, PacketKind::Error, error.to_string().as_bytes());
    }
}

#[async_trait::async_trait]
pub trait WriteBytesExt {
    async fn send_packet<T: ToBytes + Send + Sync>(&mut self, value: &T) -> tokio::io::Result<()>;
}

#[async_trait::async_trait]
impl WriteBytesExt for TcpStream {
    async fn send_packet<T: ToBytes + Send + Sync>(&mut self, value: &T) -> tokio::io::Result<()> {
        let bytes = value.to_bytes();
        self.write_all(&bytes).await
    }
}

#[async_trait::async_trait]
impl WriteBytesExt for OwnedWriteHalf {
    async fn send_packet<T: ToBytes + Send + Sync>(&mut self, value: &T) -> tokio::io::Result<()> {
        let bytes = value.to_bytes();
        self.write_all(&bytes).await
    }
}

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for Packet {
    fn to_bytes(&self) -> Vec<u8> {
        let id = i32::to_le_bytes(self.id);
        let size = i32::to_le_bytes(self.size);
        let kind = PacketKind::bytes(&self.kind);
        let mut packet = Vec::new();

        packet.extend(id);
        packet.extend(kind);
        packet.extend(size);
        packet.extend(&self.body);
        packet.extend([0x00, 0x00]);

        return packet;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bytes() {
        let bytes = [
            0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x68, 0x65,
            0x6C, 0x6C, 0x6F, 0x00, 0x00,
        ];

        if let Ok(packet) = Packet::from_bytes(&bytes) {
            assert_eq!(packet.kind, PacketKind::Broadcast);
            assert_eq!(packet.size, 15);
            assert_eq!(packet.id, 1);
            let body = String::from_utf8(packet.body.to_vec()).unwrap();
            assert_eq!(body, "hello");
        }
    }
}
