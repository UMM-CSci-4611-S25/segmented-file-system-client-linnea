use super::PacketParseError;

use std::ffi::OsString;

#[derive(Debug, PartialEq)]
pub struct Header {
    pub(crate) file_id: u8,
    pub(crate) file_name: OsString,
}

impl TryFrom<&[u8]> for Header {
    type Error = PacketParseError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 3 {
            return Err(PacketParseError::InvalidHeaderPacket);
        }
        let file_id = buffer[1];
        let file_name = OsString::from(String::from_utf8_lossy(&buffer[2..]).to_string());
        Ok(Header { file_id, file_name })
    }
}
