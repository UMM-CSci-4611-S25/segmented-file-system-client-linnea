
// Below is a version of the `main` function and some error types. This assumes
// the existence of types like `FileManager`, `Packet`, and `PacketParseError`.
// You can use this code as a starting point for the exercise, or you can
// delete it and write your own code with the same function signature.

mod file_manager;

mod packet;

use std::{
    collections::HashMap, ffi::OsString, io::{self, Write}, net::UdpSocket, os::unix::ffi::OsStringExt
};
use packet::PacketParseError;

#[derive(Debug)]
pub enum ClientError {
    IoError(std::io::Error),
    PacketParseError(packet::PacketParseError),
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::IoError(e)
    }
}

impl From<packet::PacketParseError> for ClientError {
    fn from(e: packet::PacketParseError) -> Self {
        Self::PacketParseError(e)
    }
}

fn main() -> Result<(), ClientError> {

    //let mut total_recieved = 0;
    let sock = UdpSocket::bind("0.0.0.0:7077")?;

    let remote_addr = "127.0.0.1:6014";
    sock.connect(remote_addr)?;
    let mut buf = [0; 1028];

    let _ = sock.send(&buf[..1028]);

    let mut file_manager = file_manager::FileManager::default();

    while !file_manager.received_all_packets() {
        let len = sock.recv(&mut buf)?;
        let packet: packet::Packet = buf[..len].try_into()?;
        //print!(".");
        //total_recieved += 1;
        file_manager.process_packet(packet);
        io::stdout().flush()?;
    }

    //println!("total recieved {}", total_recieved);

    file_manager.write_all_files()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_in_file_name() {
        let sparkle_heart: &[u8] = "\x00\x0CThis file is lovely 💖".as_bytes();
        let result = packet::header_packet::Header::try_from(sparkle_heart);
        assert_eq!(
            result,
            Ok(packet::header_packet::Header {
                file_id: 12,
                file_name: "This file is lovely 💖".to_string().into()
            })
        );
    }

    #[test]
    fn valid_header_packet() {
        let valid_packet: &[u8] = b"\x00\x01Hello";
        let result = packet::header_packet::Header::try_from(valid_packet);
        assert_eq!(
            result,
            Ok(packet::header_packet::Header {
                file_id: 1,
                file_name: OsString::from("Hello")
            })
        );
    }

    #[test]
    fn invalid_header_packet() {
        let invalid_packet: &[u8] = b"\x00\x01";
        let result = packet::header_packet::Header::try_from(invalid_packet);
        assert_eq!(result, Err(PacketParseError::InvalidHeaderPacket));
    }

    #[test]
    fn invalid_data_packet() {
        let invalid_packet: &[u8] = b"\x01\x01";
        let result = packet::data_packet::Data::try_from(invalid_packet);
        assert_eq!(result, Err(PacketParseError::InvalidDataPacket));
    }

    #[test]
    fn valid_data_packet() {
        let valid_packet: &[u8] = b"\x01\x01\x00\x01\x00\x02Hello";
        let result = packet::data_packet::Data::try_from(valid_packet);
        assert_eq!(
            result,
            Ok(packet::data_packet::Data {
                file_id: 1,
                packet_number: 1,
                is_last_packet: false,
                data: b"\x02Hello".to_vec()
            })
        );
    }

    #[test]
    fn test_file_manager() {
        let mut file_manager = file_manager::FileManager::default();
        let header_packet = packet::Packet::Header(packet::header_packet::Header {
            file_id: 1,
            file_name: OsString::from("test_file"),
        });
        file_manager.process_packet(header_packet);

        let data_packet = packet::Packet::Data(packet::data_packet::Data {
            file_id: 1,
            packet_number: 1,
            is_last_packet: false,
            data: b"Hello".to_vec(),
        });
        file_manager.process_packet(data_packet);

        assert_eq!(file_manager.files.len(), 1);
    }

    #[test]
    fn test_packet_parsing() {
        let header_packet: &[u8] = b"\x00\x01Hello";
        let data_packet: &[u8] = b"\x01\x01\x00\x01\x00\x02Hello";

        let header_result = packet::Packet::try_from(header_packet);
        assert!(header_result.is_ok());

        let data_result = packet::Packet::try_from(data_packet);
        assert!(data_result.is_ok());
    }
}



