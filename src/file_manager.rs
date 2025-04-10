use std::ffi::OsString;

//use crate::packet::data_packet;

//use crate::packet::header_packet;

use crate::packet;

use std::collections::HashMap;

pub struct FileManager {
    pub(crate) files: HashMap<u8, Vec<packet::Packet>>,
    pub(crate) headers: HashMap<u8, packet::Packet>,
    pub(crate) received_packets: HashMap<u8, u16>,
    pub(crate) total_packets: HashMap<u8, u16>,
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

    pub fn process_packet(&mut self, packet: packet::Packet) {
        match packet {
            packet::Packet::Header(header) => {
                self.headers.insert(header.file_id, packet::Packet::Header(header));
            }
            packet::Packet::Data(data) => {
                let file_id = data.file_id;
                let packet_number = data.packet_number;
                let is_last_packet = data.is_last_packet;


                let file = self.files.entry(data.file_id).or_insert_with(Vec::new);
                file.push(packet::Packet::Data(data));

                let received_packets = self.received_packets.entry(file_id).or_insert(0);
                *received_packets += 1;
                //println!("{} for {}", received_packets, file_id);

                if is_last_packet {
                   let total = self.total_packets.entry(file_id).or_insert(0);
                   *total = packet_number; //change to just inserting
                }

            }
        }
    }

    pub fn received_all_packets(&self) -> bool {
        if self.received_packets.is_empty() {
            return false
        }

        let iterate = self.received_packets.iter();

        let mut ids_recieved = 0;
        for  (file_id, received_packets) in iterate {
            let total_packets = match self.total_packets.get(file_id) {
                Some(num) => num,
                None => return false
            };
            ids_recieved += 1;

            //println!("Received packets for {file_id} is {received_packets} with total {}.", *total_packets);
            if *received_packets == *total_packets + 1 && *total_packets > 400{
            //println!("{} = {}, {}", received_packets, total_packets + 1, ids_recieved);
            }
            if !(*received_packets == *total_packets + 1){
                return false
            }

            if !self.headers.contains_key(file_id) {
                return false
            }
        }

        //println!("final ids recieved {}", ids_recieved);

        ids_recieved == 3
    }

    pub fn sort_and_return_data(&self, file_id: u8) -> Vec<u8> {
        let unsorted_packets = self.files.get(&file_id).unwrap();
        //println!("The number of packets for file {file_id} is {}.", unsorted_packets.len());
        let mut data_map: HashMap<u16, Vec<u8>> = HashMap::new();
        let total = self.total_packets.get(&file_id).unwrap();
        assert_eq!(*total as usize + 1, unsorted_packets.len());
        let mut whole_data: Vec<u8> = Vec::new();

        for packet in unsorted_packets {
            let packet_num: u16;
            let data_vec: Vec<u8>;
            match packet {
                packet::Packet::Header(_) => {
                    packet_num = 0;
                    data_vec = Vec::new();
                },
                packet::Packet::Data(data) => {
                    packet_num = data.packet_number;
                    data_vec = data.data.clone();
                },
            }

            data_map.insert(packet_num, data_vec);
        }

        //println!("`data_map` has size {}.", data_map.len());

        // if *total < 400{
        for i in 0..*total + 1 {
            //println!("total {} on i {}", *total, i);
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
            packet::Packet::Header(header) => file_name = header.file_name.clone(),
            packet::Packet::Data(_) => file_name = "it_went_wrong.txt".into(),
        };

        let data = self.sort_and_return_data(file_id);



        std::fs::write(file_name, data).unwrap();
    }

    pub fn write_all_files(&self) -> Result<(), std::io::Error> {
        //println!("in write all files");
        for (file_id, _file) in &self.files {
        
            self.write_file(*file_id);
            //println!("writing file id: {}", file_id);
        }
        Ok(())
    }

}
