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

#[derive(Debug)]
pub enum PacketParseError {
    InvalidHeaderPacket,
    InvalidDataPacket,

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
            // print!("test");
            let file_name = OsString::from("file_").into_vec();
            let file_name = OsString::from_vec(file_name);
            let file_name = file_name.into_string().unwrap();
            let file_name = format!("{}.bin", file_name);
            // print!("{}", file_name);
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