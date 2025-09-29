use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::utils::errors::Error;

#[repr(u8)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum TileKind {
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

impl TileKind {
    pub fn parse(value: i8) -> Option<Self> {
        match value {
            1 => Some(TileKind::Souzu1),
            2 => Some(TileKind::Souzu2),
            3 => Some(TileKind::Souzu3),
            4 => Some(TileKind::Souzu4),
            5 => Some(TileKind::Souzu5),
            6 => Some(TileKind::Souzu6),
            7 => Some(TileKind::Souzu7),
            8 => Some(TileKind::Souzu8),
            9 => Some(TileKind::Souzu9),
            11 => Some(TileKind::Pinzu1),
            12 => Some(TileKind::Pinzu2),
            13 => Some(TileKind::Pinzu3),
            14 => Some(TileKind::Pinzu4),
            15 => Some(TileKind::Pinzu5),
            16 => Some(TileKind::Pinzu6),
            17 => Some(TileKind::Pinzu7),
            18 => Some(TileKind::Pinzu8),
            19 => Some(TileKind::Pinzu9),
            21 => Some(TileKind::Manzu1),
            22 => Some(TileKind::Manzu2),
            23 => Some(TileKind::Manzu3),
            24 => Some(TileKind::Manzu4),
            25 => Some(TileKind::Manzu5),
            26 => Some(TileKind::Manzu6),
            27 => Some(TileKind::Manzu7),
            28 => Some(TileKind::Manzu8),
            29 => Some(TileKind::Manzu9),
            31 => Some(TileKind::Red),
            32 => Some(TileKind::White),
            33 => Some(TileKind::Green),
            34 => Some(TileKind::East),
            35 => Some(TileKind::West),
            36 => Some(TileKind::North),
            37 => Some(TileKind::South),
            _ => None,
        }
    }
}

impl Display for TileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            // Bamboos
            TileKind::Souzu1 => "1s",
            TileKind::Souzu2 => "2s",
            TileKind::Souzu3 => "3s",
            TileKind::Souzu4 => "4s",
            TileKind::Souzu5 => "5s",
            TileKind::Souzu6 => "6s",
            TileKind::Souzu7 => "7s",
            TileKind::Souzu8 => "8s",
            TileKind::Souzu9 => "9s",
            // Circles
            TileKind::Pinzu1 => "1p",
            TileKind::Pinzu2 => "2p",
            TileKind::Pinzu3 => "3p",
            TileKind::Pinzu4 => "4p",
            TileKind::Pinzu5 => "5p",
            TileKind::Pinzu6 => "6p",
            TileKind::Pinzu7 => "7p",
            TileKind::Pinzu8 => "8p",
            TileKind::Pinzu9 => "9p",
            // Characters
            TileKind::Manzu1 => "1m",
            TileKind::Manzu2 => "2m",
            TileKind::Manzu3 => "3m",
            TileKind::Manzu4 => "4m",
            TileKind::Manzu5 => "5m",
            TileKind::Manzu6 => "6m",
            TileKind::Manzu7 => "7m",
            TileKind::Manzu8 => "8m",
            TileKind::Manzu9 => "9m",
            // Dragons
            TileKind::Red => "Red",
            TileKind::White => "White",
            TileKind::Green => "Green",
            // Winds
            TileKind::East => "East",
            TileKind::West => "West",
            TileKind::North => "North",
            TileKind::South => "South",
        };
        write!(f, "{}", s)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Tile {
    pub kind: TileKind,
    pub copy: u8, // 0-3
}

impl Tile {
    pub fn from_bytes(t: u8, c: u8) -> Result<Tile, Error> {
        if let Some(kind) = TileKind::parse(t as i8) {
            return Ok(Self { kind, copy: c });
        }

        return Err(Error::TileParsingFailed);
    }
}
