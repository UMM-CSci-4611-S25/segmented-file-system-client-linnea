
// Below is a version of the `main` function and some error types. This assumes
// the existence of types like `FileManager`, `Packet`, and `PacketParseError`.
// You can use this code as a starting point for the exercise, or you can
// delete it and write your own code with the same function signature.



use std::{
    ffi::OsString, io::{self, Write}, net::UdpSocket, os::unix::ffi::OsStringExt
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

#[derive(Debug)]
pub enum PacketParseError {

}

pub enum Packet {
    Header(Header),
    Data(Data)
}

pub struct Header {
    file_id: u8,
    file_name: OsString
}

pub struct Data {
    file_id: u8,
    packet_number: u16,
    is_last_packet: bool,
    data: Vec<u8>
}


impl TryFrom<&[u8]> for Packet {
    type Error = PacketParseError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {

        if buffer[0] % 2 == 0 {  //header packet

            if buffer.len < 4

            let file_name= OsString::from_vec(buffer[2..].to_vec());

            let file_id = buffer[1];

            Ok(Packet::Header(Header { file_id, file_name }))
        }
        else {
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



