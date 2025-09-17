use std::fmt::{self, Display};

use crate::utils::errors::Error;

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
    pub fn parse(value: i8) -> Option<Self> {
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

impl Display for TileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            // Bamboos
            TileType::Souzu1 => "1s",
            TileType::Souzu2 => "2s",
            TileType::Souzu3 => "3s",
            TileType::Souzu4 => "4s",
            TileType::Souzu5 => "5s",
            TileType::Souzu6 => "6s",
            TileType::Souzu7 => "7s",
            TileType::Souzu8 => "8s",
            TileType::Souzu9 => "9s",
            // Circles
            TileType::Pinzu1 => "1p",
            TileType::Pinzu2 => "2p",
            TileType::Pinzu3 => "3p",
            TileType::Pinzu4 => "4p",
            TileType::Pinzu5 => "5p",
            TileType::Pinzu6 => "6p",
            TileType::Pinzu7 => "7p",
            TileType::Pinzu8 => "8p",
            TileType::Pinzu9 => "9p",
            // Characters
            TileType::Manzu1 => "1m",
            TileType::Manzu2 => "2m",
            TileType::Manzu3 => "3m",
            TileType::Manzu4 => "4m",
            TileType::Manzu5 => "5m",
            TileType::Manzu6 => "6m",
            TileType::Manzu7 => "7m",
            TileType::Manzu8 => "8m",
            TileType::Manzu9 => "9m",
            // Dragons
            TileType::Red => "Red",
            TileType::White => "White",
            TileType::Green => "Green",
            // Winds
            TileType::East => "East",
            TileType::West => "West",
            TileType::North => "North",
            TileType::South => "South",
        };
        write!(f, "{}", s)
    }
}

pub struct Tile {
    pub kind: TileType,
    pub copy: u8, // 0-3
}

impl Tile {
    pub fn from_bytes(t: u8, c: u8) -> Result<Tile, Error> {
        if let Some(kind) = TileType::parse(t as i8) {
            return Ok(Self { kind, copy: c });
        }

        return Err(Error::TileParsingFailed);
    }
}
