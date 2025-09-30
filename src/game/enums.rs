use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::utils::errors::Error;

#[repr(i8)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Seat {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

impl Seat {
    pub fn to_string(&self) -> String {
        match self {
            Self::North => "north".to_string(),
            Self::South => "south".to_string(),
            Self::East => "east".to_string(),
            Seat::West => "west".to_string(),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum PlayerStatus {
    WAITING,
    DRAW,
    DISCARD,
    READY,
}

#[derive(PartialEq, Eq)]
pub enum Action {
    DRAW = 0,
    DISCARD = 1,
    KAN = 2,
    PON = 3,
    CHI = 4,
    RON = 5,
    TSUMO = 6,
}

impl Action {
    pub fn bytes(&self) -> [u8; 4] {
        let leading = match self {
            Self::DRAW => 0x00,
            Self::DISCARD => 0x01,
            Self::KAN => 0x02,
            Self::PON => 0x03,
            Self::CHI => 0x04,
            Self::RON => 0x05,
            Self::TSUMO => 0x06,
        };

        return [leading, 0x00, 0x00, 0x00];
    }

    pub fn get(value: u8) -> Option<Action> {
        match value {
            0 => Some(Action::DRAW),
            1 => Some(Action::DISCARD),
            2 => Some(Action::KAN),
            3 => Some(Action::PON),
            4 => Some(Action::CHI),
            5 => Some(Action::RON),
            6 => Some(Action::TSUMO),
            _ => None,
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DRAW => write!(f, "DRAW"),
            Self::DISCARD => write!(f, "DISCARD"),
            Self::KAN => write!(f, "KAN"),
            Self::PON => write!(f, "PON"),
            Self::CHI => write!(f, "CHI"),
            Self::RON => write!(f, "RON"),
            Self::TSUMO => write!(f, "TSUMO"),
        }
    }
}

#[repr(i8)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum TileKind {
    // Bamboos
    Souzu1 = 11,
    Souzu2 = 12,
    Souzu3 = 13,
    Souzu4 = 14,
    Souzu5 = 15,
    Souzu6 = 16,
    Souzu7 = 17,
    Souzu8 = 18,
    Souzu9 = 19,
    // Circles
    Pinzu1 = 21,
    Pinzu2 = 22,
    Pinzu3 = 23,
    Pinzu4 = 24,
    Pinzu5 = 25,
    Pinzu6 = 26,
    Pinzu7 = 27,
    Pinzu8 = 28,
    Pinzu9 = 29,
    // Characters
    Manzu1 = 31,
    Manzu2 = 32,
    Manzu3 = 33,
    Manzu4 = 34,
    Manzu5 = 35,
    Manzu6 = 36,
    Manzu7 = 37,
    Manzu8 = 38,
    Manzu9 = 39,
    // Dragons
    Red = 41,
    White = 42,
    Green = 43,
    // Winds
    East = 44,
    West = 45,
    North = 46,
    South = 47,
}

impl From<TileKind> for i8 {
    fn from(tile: TileKind) -> Self {
        tile as i8
    }
}

impl TileKind {
    pub fn parse(value: i8) -> Option<Self> {
        match value {
            11 => Some(TileKind::Souzu1),
            12 => Some(TileKind::Souzu2),
            13 => Some(TileKind::Souzu3),
            14 => Some(TileKind::Souzu4),
            15 => Some(TileKind::Souzu5),
            16 => Some(TileKind::Souzu6),
            17 => Some(TileKind::Souzu7),
            18 => Some(TileKind::Souzu8),
            19 => Some(TileKind::Souzu9),
            21 => Some(TileKind::Pinzu1),
            22 => Some(TileKind::Pinzu2),
            23 => Some(TileKind::Pinzu3),
            24 => Some(TileKind::Pinzu4),
            25 => Some(TileKind::Pinzu5),
            26 => Some(TileKind::Pinzu6),
            27 => Some(TileKind::Pinzu7),
            28 => Some(TileKind::Pinzu8),
            29 => Some(TileKind::Pinzu9),
            31 => Some(TileKind::Manzu1),
            32 => Some(TileKind::Manzu2),
            33 => Some(TileKind::Manzu3),
            34 => Some(TileKind::Manzu4),
            35 => Some(TileKind::Manzu5),
            36 => Some(TileKind::Manzu6),
            37 => Some(TileKind::Manzu7),
            38 => Some(TileKind::Manzu8),
            39 => Some(TileKind::Manzu9),
            41 => Some(TileKind::Red),
            42 => Some(TileKind::White),
            43 => Some(TileKind::Green),
            44 => Some(TileKind::East),
            45 => Some(TileKind::West),
            46 => Some(TileKind::North),
            47 => Some(TileKind::South),
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
    pub copy: u8, // 0-3
    pub kind: TileKind,
}

impl Tile {
    pub fn from_bytes(t: u8, c: u8) -> Result<Tile, Error> {
        if let Some(kind) = TileKind::parse(t as i8) {
            return Ok(Self { kind, copy: c });
        }

        return Err(Error::TileParsingFailed);
    }
}
