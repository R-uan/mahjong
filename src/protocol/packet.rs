use std::io::Error;

pub enum PacketType {
    Ping = 0x00,
    Reconnection = 0x02,
    Authentication = 0x01,
    GameAction = 0x03,
}

pub struct Packet {
    pub packet_id: i32,
    pub packet_size: i32,
    pub packet_body: Box<[u8]>,
    pub packet_type: PacketType,
}

impl Packet {
    pub fn from_bytes(bytes: &[u8]) -> Result<Packet, Error> {
        todo!()
    }
}
