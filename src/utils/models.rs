use serde::{Deserialize, Serialize};

use crate::{
    game::enums::{Action, Tile, TileKind},
    protocol::packet::{Packet, PacketKind},
    utils::errors::Error,
};

#[derive(Serialize, Deserialize)]
pub struct JoinRequest {
    pub id: i32,
    pub alias: String,
}

impl JoinRequest {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let id = bytes
            .get(0..=7)
            .ok_or(Error::ConnectionFailed(54))?
            .try_into()
            .ok()
            .map(i32::from_le_bytes)
            .ok_or(Error::ConnectionFailed(54))?;
        let alias_bytes = bytes.get(8..).ok_or(Error::ConnectionFailed(54))?;
        let alias =
            String::from_utf8(alias_bytes.into()).map_err(|_| Error::ConnectionFailed(54))?;
        return Ok(JoinRequest { id, alias });
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::models::JoinRequest;

    #[test]
    fn test_parse() {
        let bytes: &[u8] = &[
            0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x42, 0x75, 0x6E, 0x6E, 0x79,
        ];
        let parse = JoinRequest::parse(bytes);
        assert!(parse.is_ok());
        assert_eq!(parse.unwrap().alias, "Bunny");
    }
}

#[derive(Serialize, Deserialize)]
pub struct Discard {
    pub player_id: i32,
    pub tile_copy: u8,
    pub tile_kind: TileKind,
}

impl Discard {
    pub fn broadcast(id: i32, pid: i32, target: Tile) -> Packet {
        let broadcast = Discard {
            player_id: pid,
            tile_kind: target.kind,
            tile_copy: target.copy,
        };
        match serde_cbor::to_vec(&broadcast) {
            Err(_) => Packet::error(id, Error::InternalError),
            Ok(bytes) => {
                let mut body: Vec<u8> = Vec::new();
                body.extend_from_slice(&Action::DISCARD.bytes());
                body.extend_from_slice(&bytes);
                Packet::create(id, PacketKind::Broadcast, &body.into_boxed_slice())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Draw {
    pub player_id: i32,
    pub tile_kind: TileKind,
    pub tile_copy: u8,
}

impl Draw {
    pub fn broadcast(id: i32, pid: i32, target: Tile) -> Packet {
        let broadcast = Discard {
            player_id: pid,
            tile_kind: target.kind,
            tile_copy: target.copy,
        };
        match serde_cbor::to_vec(&broadcast) {
            Err(_) => Packet::error(id, Error::InternalError),
            Ok(bytes) => {
                let mut body: Vec<u8> = Vec::new();
                body.extend_from_slice(&Action::DISCARD.bytes());
                body.extend_from_slice(&bytes);
                Packet::create(id, PacketKind::Broadcast, &body.into_boxed_slice())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MeldFlags {
    pub pid: i32,
    pub ron: bool,
    pub pon: bool,
    pub chi: bool,
    pub kan: bool,
}

impl MeldFlags {
    pub fn create(pid: i32, flags: mlua::Table) -> Result<Self, Error> {
        Ok(Self {
            pid,
            chi: flags.get("chi").map_err(|_| Error::InternalError)?,
            pon: flags.get("pon").map_err(|_| Error::InternalError)?,
            kan: flags.get("kan").map_err(|_| Error::InternalError)?,
            ron: flags.get("ron").map_err(|_| Error::InternalError)?,
        })
    }
}
