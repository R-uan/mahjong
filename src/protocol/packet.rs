use crate::utils::errors::Error;

#[derive(Debug, PartialEq)]
pub enum PacketKind {
    Ping = 0,
    Reconnection = 2,
    Authentication = 1,
    GameAction = 3,
    ActionDone = 4,
    ActionFail = 5,
    Error = 6,
}

impl PacketKind {
    pub fn from_byte(byte: u32) -> Option<Self> {
        match byte as i32 {
            0 => Some(Self::Ping),
            1 => Some(Self::Authentication),
            2 => Some(Self::Reconnection),
            3 => Some(Self::GameAction),
            _ => None,
        }
    }

    pub fn get_bytes(t: PacketKind) -> [u8; 4] {
        match t {
            PacketKind::Ping => [0x00, 0x00, 0x00, 0x00],
            PacketKind::Authentication => [0x01, 0x00, 0x00, 0x00],
            PacketKind::Reconnection => [0x02, 0x00, 0x00, 0x00],
            PacketKind::GameAction => [0x03, 0x00, 0x00, 0x00],
            PacketKind::ActionDone => [0x04, 0x00, 0x00, 0x00],
            PacketKind::ActionFail => [0x05, 0x00, 0x00, 0x00],
            PacketKind::Error => [0x06, 0x00, 0x00, 0x00],
        }
    }
}

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

    pub fn create(id: i32, packet_type: PacketKind, body: &[u8]) -> Packet {
        Self {
            kind: packet_type,
            id,
            size: (8 + 2 + body.len()) as i32,
            body: body.to_vec().into_boxed_slice(),
        }
    }

    pub fn wrap(&self) -> Box<[u8]> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bytes() {
        let bytes = [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x68, 0x65,
            0x6C, 0x6C, 0x6F, 0x00, 0x00,
        ];

        if let Ok(packet) = Packet::from_bytes(&bytes) {
            assert_eq!(packet.kind, PacketKind::Ping);
            assert_eq!(packet.size, 15);
            assert_eq!(packet.id, 1);
            let body = String::from_utf8(packet.body.to_vec()).unwrap();
            assert_eq!(body, "hello");
        }
    }
}
