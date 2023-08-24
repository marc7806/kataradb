use std::fmt::format;
use std::fs::read;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpListener, TcpStream};

// Implement I/O Multiplexing, single-threaded event-loop

const PORT: i16 = 9977;
const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)); // IPv4 addresses are 32bit sized

fn main() {
    println!("Starting kataradb");

    let listener = TcpListener::bind(format!("{ADDRESS}:{PORT}")).expect("Can not create TCP listener");

    println!("Waiting for connections...");

    // accept TCP connections and process them sequentially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Connected to new client");
                handle_connection(stream)
            }
            Err(e) => {
                eprintln!("Error handling stream: {e}");
                continue;
            }
        }
    }
}

fn handle_connection(stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    for line in reader.lines() {
        let receive_message = line.unwrap();
        println!("{}", receive_message);
        writer.write(format!("Echo: {} \n", receive_message).as_bytes()).expect("Can not send answer");
        writer.flush().expect("Can not flush message on BufWriter");
    }
}
