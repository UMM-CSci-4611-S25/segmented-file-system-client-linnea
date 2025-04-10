use std::ffi::OsString;

pub mod header_packet;

pub mod data_packet;

#[derive(Debug, PartialEq)]
pub enum PacketParseError {
    InvalidHeaderPacket,
    InvalidDataPacket,
}

#[derive(Debug)]
pub enum Packet {
    Header(header_packet::Header),
    Data(data_packet::Data)
}

impl TryFrom<&[u8]> for Packet {
    type Error = PacketParseError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {

        if buffer[0] % 2 == 0 {  //header packet

            if buffer.len() < 4 {
                return Err(PacketParseError::InvalidHeaderPacket);
            }

            let file_name = OsString::from(String::from_utf8_lossy(&buffer[2..]).to_string());

            let file_id = buffer[1];

            Ok(Packet::Header(header_packet::Header { file_id, file_name }))
        }
        else {

            if buffer.len() < 6 {
                return Err(PacketParseError::InvalidDataPacket);
            }
            let file_id = buffer[1];
            let packet_number_bytes = [buffer[2], buffer[3]];
            let packet_number = u16::from_be_bytes(packet_number_bytes);

            let is_last_packet = buffer[0] % 4 == 3;
            let data = buffer[4..].to_vec();

            Ok(Packet::Data(data_packet::Data { file_id, packet_number, is_last_packet, data}))

        }

    }
}
