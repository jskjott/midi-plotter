extern crate coremidi;
extern crate serialport;

use std::time::{Duration, SystemTime};
use std::env;
use serialport::prelude::*;
use std::collections::HashMap;

fn send_commands(cmds: &Vec<String>, port: &mut Box<dyn SerialPort>) {
    let cmds: Vec<Vec<u8>> = cmds.iter().map(|x| x.clone().into_bytes()).collect();
    let mut next_cmd = vec![];
    let mut chunks: Vec<Vec<u8>> = vec![];
    for cmd in cmds.iter() {
        if next_cmd.len() + cmd.len() < 57 {
            next_cmd.append(&mut cmd.clone());
        } else {
            chunks.push(next_cmd);
            next_cmd = cmd.to_vec();
        }
    }
    chunks.push(next_cmd);
    for chunk in chunks {
        port.write(&chunk);
        port.write(b"OA;");
        let mut c = 0;
        while c != 13 {
            let mut v = vec![0];
            port.read(v.as_mut_slice());
            c = v[0];
        }
    }
}

fn main() {

    let start_time = SystemTime::now();
    let mut active_notes: HashMap<u8, SystemTime> = HashMap::new();

    let s = SerialPortSettings {
        baud_rate: 9600,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1000),
    };

    let mut port = serialport::open_with_settings("/dev/tty.usbserial", &s).unwrap();
    port.write(b"IN;");

    let source_index = 1;

    let source = coremidi::Source::from_index(source_index).unwrap();
    println!("Source display name: {}", source.display_name().unwrap());

    let client = coremidi::Client::new("example-client").unwrap();

    let callback = move |packet_list: &coremidi::PacketList| {
        for packet in packet_list.iter() {
            if packet.data().len() == 3 {

                if active_notes.contains_key(&packet.data()[1]) {
                    
                    let time = active_notes.get(&packet.data()[1]).cloned();
                    let time = time.unwrap();

                    match time.elapsed() {
                       Ok(elapsed) => {
                           println!("{:?} {:?}", &packet.data()[1], elapsed.as_millis());
                           let key = packet.data()[1];
                           let time = elapsed.as_millis();

                           let time_since_start = match start_time.elapsed() {
                                Ok(elapsed) => {
                                    println!("{:?}", elapsed.as_millis());
                                    elapsed.as_millis()
                                }
                                Err(e) => {
                                   println!("Error: {:?}", e);
                                   0
                               }
                           };

                           let y: u32 = 50 + 74 * key as u32;
                           let x: u32 = (00.16 * (time_since_start as f32) + 50 as f32) as u32;
                           let width: u32 = (00.16 * (time as f32)) as u32;
                           let height: u32 = 74;

                           let mut square: Vec<String> = vec![];

                           square.push("SP1;".to_string());
                           square.push(format!("PA{},{};", x, y));
                           square.push("PD;".to_string());
                           square.push(format!("PA{},{};", x + width, y));
                           square.push(format!("PA{},{};", x + width, y + height));
                           square.push(format!("PA{},{};", x, y + height));
                           square.push(format!("PA{},{};", x, y));
                           square.push("PU;".to_string());

                           send_commands(&square.iter().map(|x| x.to_string()).collect(), &mut port);
                       }
                       Err(e) => {
                           println!("Error: {:?}", e);
                       }
                   };

                    active_notes.remove(&packet.data()[1]);
                } else {
                    let timestamp = SystemTime::now();
                    active_notes.insert(packet.data()[1], timestamp);    
                }
            }
        };
    };

    let input_port = client.input_port("example-port", callback).unwrap();
    input_port.connect_source(&source).unwrap();

    let mut input_line = String::new();
    std::io::stdin().read_line(&mut input_line).ok().expect("Failed to read line");

    input_port.disconnect_source(&source).unwrap();
}
