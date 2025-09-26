pub enum Setup {
    Connection = 1,
    Reconnection = 2,
    Initialization = 3,
    Ready = 4,
}

impl Setup {
    pub fn from(bytes: &[u8]) -> Option<Self> {
        match u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i32 {
            1 => Some(Self::Connection),
            2 => Some(Self::Reconnection),
            3 => Some(Self::Initialization),
            4 => Some(Self::Ready),
            _ => None,
        }
    }

    pub fn bytes(&self) -> [u8; 4] {
        match self {
            Self::Connection => [0x01, 0x00, 0x00, 0x00],
            Self::Reconnection => [0x02, 0x00, 0x00, 0x00],
            Self::Initialization => [0x03, 0x00, 0x00, 0x00],
            Self::Ready => [0x04, 0x00, 0x00, 0x00],
        }
    }
}
