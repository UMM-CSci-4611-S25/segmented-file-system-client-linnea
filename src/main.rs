
// Below is a version of the `main` function and some error types. This assumes
// the existence of types like `FileManager`, `Packet`, and `PacketParseError`.
// You can use this code as a starting point for the exercise, or you can
// delete it and write your own code with the same function signature.



use std::{
    collections::HashMap, ffi::OsString, io::{self, Write}, net::UdpSocket, os::unix::ffi::OsStringExt
};

#[derive(Debug)]
pub enum ClientError {
    IoError(std::io::Error),
    PacketParseError(PacketParseError),
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::IoError(e)
    }
}

impl From<PacketParseError> for ClientError {
    fn from(e: PacketParseError) -> Self {
        Self::PacketParseError(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketParseError {
    InvalidHeaderPacket,
    InvalidDataPacket,
}

#[derive(Debug)]
pub enum Packet {
    Header(Header),
    Data(Data)
}

#[derive(Debug, PartialEq)]
pub struct Header {
    file_id: u8,
    file_name: OsString,
}

impl TryFrom<&[u8]> for Header {
    type Error = PacketParseError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 3 {
            return Err(PacketParseError::InvalidHeaderPacket);
        }
        let file_id = buffer[1];
        let file_name = OsString::from_vec(buffer[2..].to_vec());
        Ok(Header { file_id, file_name })
    }
}

#[derive(Debug, PartialEq)]
pub struct Data {
    file_id: u8,
    packet_number: u16,
    is_last_packet: bool,
    data: Vec<u8>
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

pub struct FileManager {
    files: HashMap<u8, Vec<u8>>,
    received_packets: HashMap<u8, u16>,
    total_packets: HashMap<u8, u16>,
}

impl FileManager {
    pub fn default() -> Self {
        Self {
            files: HashMap::new(),
            received_packets: HashMap::new(),
            total_packets: HashMap::new(),
        }
    }

    pub fn process_packet(&mut self, packet: Packet) {
        match packet {
            Packet::Header(header) => {
                self.total_packets.insert(header.file_id, 0);
                self.received_packets.insert(header.file_id, 0);
            }
            Packet::Data(data) => {
                let file_id = data.file_id;
                let packet_number = data.packet_number;
                let is_last_packet = data.is_last_packet;
                let data = data.data;

                let file = self.files.entry(file_id).or_insert_with(Vec::new);
                file.extend(data);

                let received_packets = self.received_packets.get_mut(&file_id).unwrap();
                *received_packets += 1;

                if is_last_packet {
                    let total_packets = self.total_packets.get(&file_id).unwrap();
                    if *received_packets == *total_packets {
                        self.write_file(file_id);
                    }
                }
            }
        }
    }

    pub fn received_all_packets(&self) -> bool {
        self.received_packets.iter().all(|(file_id, received_packets)| {
            let total_packets = self.total_packets.get(file_id).unwrap();
            received_packets == total_packets
        })
    }

    pub fn write_file(&self, file_id: u8) {
        let file = self.files.get(&file_id).unwrap();
        let file_name = OsString::from("file_").into_vec();
        let file_name = OsString::from_vec(file_name);
        let file_name = file_name.into_string().unwrap();
        let file_name = format!("{}.bin", file_name);
        std::fs::write(file_name, file).unwrap();
    }

    pub fn write_all_files(&self) -> Result<(), std::io::Error> {
        for (file_id, file) in &self.files {
            let file_name = OsString::from("file_").into_vec();
            let file_name = OsString::from_vec(file_name);
            let file_name = file_name.into_string().unwrap();
            let file_name = format!("{}.bin", file_name);
            std::fs::write(file_name, file)?;
        }
        Ok(())
    }

}

impl TryFrom<&[u8]> for Packet {
    type Error = PacketParseError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {

        if buffer[0] % 2 == 0 {  //header packet

            if buffer.len() < 4 {
                return Err(PacketParseError::InvalidHeaderPacket);
            }

            let file_name= OsString::from_vec(buffer[2..].to_vec());

            let file_id = buffer[1];

            Ok(Packet::Header(Header { file_id, file_name }))
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

            Ok(Packet::Data(Data { file_id, packet_number, is_last_packet, data}))

        }

    }
}



fn main() -> Result<(), ClientError> {
    let sock = UdpSocket::bind("0.0.0.0:7077")?;

    let remote_addr = "127.0.0.1:6014";
    sock.connect(remote_addr)?;
    let mut buf = [0; 1028];

    let _ = sock.send(&buf[..1028]);

    let mut file_manager = FileManager::default();

    while !file_manager.received_all_packets() {
        let len = sock.recv(&mut buf)?;
        let packet: Packet = buf[..len].try_into()?;
        print!(".");
        io::stdout().flush()?;
        file_manager.process_packet(packet);
    }

    file_manager.write_all_files()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_in_file_name() {
        let sparkle_heart: &[u8] = "\x00\x0CThis file is lovely 💖".as_bytes();
        let result = Header::try_from(sparkle_heart);
        assert_eq!(
            result,
            Ok(Header {
                file_id: 12,
                file_name: "This file is lovely 💖".to_string().into()
            })
        );
    }

    #[test]
    fn valid_header_packet() {
        let valid_packet: &[u8] = b"\x00\x01Hello";
        let result = Header::try_from(valid_packet);
        assert_eq!(
            result,
            Ok(Header {
                file_id: 1,
                file_name: OsString::from("Hello")
            })
        );
    }

    #[test]
    fn invalid_header_packet() {
        let invalid_packet: &[u8] = b"\x00\x01";
        let result = Header::try_from(invalid_packet);
        assert_eq!(result, Err(PacketParseError::InvalidHeaderPacket));
    }

    #[test]
    fn invalid_data_packet() {
        let invalid_packet: &[u8] = b"\x01\x01";
        let result = Data::try_from(invalid_packet);
        assert_eq!(result, Err(PacketParseError::InvalidDataPacket));
    }

    #[test]
    fn valid_data_packet() {
        let valid_packet: &[u8] = b"\x01\x01\x00\x01\x00\x02Hello";
        let result = Data::try_from(valid_packet);
        assert_eq!(
            result,
            Ok(Data {
                file_id: 1,
                packet_number: 1,
                is_last_packet: false,
                data: b"\x02Hello".to_vec()
            })
        );
    }

    #[test]
    fn test_file_manager() {
        let mut file_manager = FileManager::default();
        let header_packet = Packet::Header(Header {
            file_id: 1,
            file_name: OsString::from("test_file"),
        });
        file_manager.process_packet(header_packet);

        let data_packet = Packet::Data(Data {
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

        let header_result = Packet::try_from(header_packet);
        assert!(header_result.is_ok());

        let data_result = Packet::try_from(data_packet);
        assert!(data_result.is_ok());
    }
}



