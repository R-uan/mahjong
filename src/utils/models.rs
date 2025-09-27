use serde::{Deserialize, Serialize};

use crate::utils::errors::Error;

#[derive(Serialize, Deserialize)]
pub struct JoinRequest {
    pub id: u64,
    pub alias: String,
}

impl JoinRequest {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let id = bytes
            .get(0..=7)
            .ok_or(Error::ConnectionFailed(54))?
            .try_into()
            .ok()
            .map(u64::from_le_bytes)
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
