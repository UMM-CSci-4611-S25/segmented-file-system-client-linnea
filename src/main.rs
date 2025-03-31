
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
    files: HashMap<u8, Vec<Packet>>,
    headers: HashMap<u8, Packet>,
    received_packets: HashMap<u8, u16>,
    total_packets: HashMap<u8, u16>,
}

impl FileManager {
    pub fn default() -> Self {
        Self {
            files: HashMap::new(),
            headers: HashMap::new(),
            received_packets: HashMap::new(),
            total_packets: HashMap::new(),
        }
    }

    pub fn process_packet(&mut self, packet: Packet) {
        match packet {
            Packet::Header(header) => {
                self.headers.insert(header.file_id, Packet::Header(header));
            }
            Packet::Data(data) => {
                let file_id = data.file_id;
                let packet_number = data.packet_number;
                let is_last_packet = data.is_last_packet;


                let file = self.files.entry(data.file_id).or_insert_with(Vec::new);
                file.push(Packet::Data(data));

                let received_packets = self.received_packets.entry(file_id).or_insert(0);
                *received_packets += 1;
                println!("{} for {}", received_packets, file_id);

                if is_last_packet {
                   let total = self.total_packets.entry(file_id).or_insert(0);
                   *total = packet_number;
                }

            }
        }
    }

    pub fn received_all_packets(&self) -> bool {
        if self.received_packets.is_empty() {
            return false
        }
        // let mut ids_recieved = 0;
        // self.received_packets.iter().all(|(file_id, received_packets)| {
        //     let total_packets = match self.total_packets.get(file_id) {
        //         Some(num) => num,
        //         None => return false
        //     };
        //     ids_recieved += 1;
        //     // if total_packets == &0 { //should return false in match statement but does not
        //     //     return false
        //     // }
        //     if *received_packets == *total_packets + 1 {
        //     println!("{} = {}, {}", received_packets, total_packets + 1, ids_recieved);
        //     }
        //     *received_packets == *total_packets + 1 && ids_recieved == 3
        // })

        let iterate = self.received_packets.iter();

        let mut ids_recieved = 0;
        for  (file_id, received_packets) in iterate {
            let total_packets = match self.total_packets.get(file_id) {
                Some(num) => num,
                None => return false
            };
            ids_recieved += 1;

            if *received_packets == *total_packets + 1 && *total_packets > 400{
            println!("{} = {}, {}", received_packets, total_packets + 1, ids_recieved);
            }
            if !*received_packets == *total_packets + 1 || !self.headers.contains_key(file_id){
                return false
            }
        }

        println!("final ids recieved {}", ids_recieved);

        ids_recieved == 3
    }

    pub fn sort_and_return_data(&self, file_id: u8) -> Vec<u8> {
        let unsorted_packets = self.files.get(&file_id).unwrap();
        let mut data_map: HashMap<u16, Vec<u8>> = HashMap::new();
        let total = self.total_packets.get(&file_id).unwrap();
        let mut whole_data: Vec<u8> = Vec::new();

        for packet in unsorted_packets {
            let packet_num: u16;
            let data_vec: Vec<u8>;
            match packet {
                Packet::Header(_) => {
                    packet_num = 0;
                    data_vec = Vec::new();
                },
                Packet::Data(data) => {
                    packet_num = data.packet_number;
                    data_vec = data.data.clone();
                },
            }

            data_map.insert(packet_num, data_vec);

        }

        // if *total < 400{
        for i in 0..*total + 1 {
            println!("total {} on i {}", *total, i);
            let data_part = data_map.get_mut(&i).unwrap();
            whole_data.append(data_part);
        // }
    }


       
        whole_data
        
    }

    pub fn write_file(&self, file_id: u8) {
        
        let name_packet = self.headers.get(&file_id).unwrap();
        let file_name: OsString;
        
        match name_packet {
            Packet::Header(header) => file_name = header.file_name.clone(),
            Packet::Data(_) => file_name = "it_went_wrong.txt".into(),
        };

        let data = self.sort_and_return_data(file_id);



        std::fs::write(file_name, data).unwrap();
    }

    pub fn write_all_files(&self) -> Result<(), std::io::Error> {
        println!("in write all files");
        for (file_id, _file) in &self.files {
            
            self.write_file(*file_id);
            println!("writing file id: {}", file_id);
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

    let mut total_recieved = 0;
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
        total_recieved += 1;
        file_manager.process_packet(packet);
        io::stdout().flush()?;
    }

    println!("total recieved {}", total_recieved);

    file_manager.write_all_files()?;

    Ok(())
}



