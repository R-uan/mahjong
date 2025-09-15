#[derive(PartialEq, Eq)]
pub enum TileType {
    // Bamboos
    Souzu1 = 1,
    Souzu2 = 2,
    Souzu3 = 3,
    Souzu4 = 4,
    Souzu5 = 5,
    Souzu6 = 6,
    Souzu7 = 7,
    Souzu8 = 8,
    Souzu9 = 9,
    // Circles
    Pinzu1 = 11,
    Pinzu2 = 12,
    Pinzu3 = 13,
    Pinzu4 = 14,
    Pinzu5 = 15,
    Pinzu6 = 16,
    Pinzu7 = 17,
    Pinzu8 = 18,
    Pinzu9 = 19,
    // Characters
    Manzu1 = 21,
    Manzu2 = 22,
    Manzu3 = 23,
    Manzu4 = 24,
    Manzu5 = 25,
    Manzu6 = 26,
    Manzu7 = 27,
    Manzu8 = 28,
    Manzu9 = 29,
    // Dragons
    Red = 31,
    White = 32,
    Green = 33,
    // Winds
    East = 34,
    West = 35,
    North = 36,
    South = 37,
}

impl TileType {
    pub fn from(value: i32) -> Option<Self> {
        match value {
            1 => Some(TileType::Souzu1),
            2 => Some(TileType::Souzu2),
            3 => Some(TileType::Souzu3),
            4 => Some(TileType::Souzu4),
            5 => Some(TileType::Souzu5),
            6 => Some(TileType::Souzu6),
            7 => Some(TileType::Souzu7),
            8 => Some(TileType::Souzu8),
            9 => Some(TileType::Souzu9),
            11 => Some(TileType::Pinzu1),
            12 => Some(TileType::Pinzu2),
            13 => Some(TileType::Pinzu3),
            14 => Some(TileType::Pinzu4),
            15 => Some(TileType::Pinzu5),
            16 => Some(TileType::Pinzu6),
            17 => Some(TileType::Pinzu7),
            18 => Some(TileType::Pinzu8),
            19 => Some(TileType::Pinzu9),
            21 => Some(TileType::Manzu1),
            22 => Some(TileType::Manzu2),
            23 => Some(TileType::Manzu3),
            24 => Some(TileType::Manzu4),
            25 => Some(TileType::Manzu5),
            26 => Some(TileType::Manzu6),
            27 => Some(TileType::Manzu7),
            28 => Some(TileType::Manzu8),
            29 => Some(TileType::Manzu9),
            31 => Some(TileType::Red),
            32 => Some(TileType::White),
            33 => Some(TileType::Green),
            34 => Some(TileType::East),
            35 => Some(TileType::West),
            36 => Some(TileType::North),
            37 => Some(TileType::South),
            _ => None,
        }
    }
}

pub struct Tile {
    pub kind: TileType,
    pub copy: u8, // 0-3
}
