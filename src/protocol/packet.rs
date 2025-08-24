pub struct Packet {
    pub length: u16,
    pub checksum: u16,
    pub packet_type: u8,
    pub payload: Box<[u8]>,
}

impl Packet {
    pub fn from_bytes(bytes: &[u8]) -> Packet {
        Packet {
            length: 0,
            checksum: 0,
            packet_type: 0,
            payload: Box::new([0; 0]),
        }
    }
}