
use super::PacketParseError;

use std::ffi::OsString;

    #[derive(Debug, PartialEq)]
    pub struct Data {
        pub(crate) file_id: u8,
        pub(crate) packet_number: u16,
        pub(crate) is_last_packet: bool,
        pub(crate) data: Vec<u8>
    }

    impl TryFrom<&[u8]> for Data {
        type Error = PacketParseError;

        fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
            if buffer.len() < 6 {
                return Err(PacketParseError::InvalidDataPacket);
            }
            let file_id = buffer[1];
            let packet_number_bytes = [buffer[2], buffer[3]];
            let packet_number = u16::from_be_bytes(packet_number_bytes);
            let is_last_packet = buffer[4] == 1;
            let data = buffer[5..].to_vec();

            Ok(Data {
                file_id,
                packet_number,
                is_last_packet,
                data,
            })
        }
    }