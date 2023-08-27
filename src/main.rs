use std::io::{BufRead, Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpListener, TcpStream};

use crate::resp::RESPParser;

pub mod resp;

// Implement I/O Multiplexing, single-threaded event-loop
const PORT: i16 = 9977;
const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)); // IPv4 addresses are 32bit sized
const TEMP_BUFFER_SIZE: usize = 512;

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

// Architecture
// Parser holds temporary buffer in which it reads max X bytes from the stream (x is configurable)
// Reading stops after \r\n is found
// then decoding of sequence starts
// then you can repeat reading bytes from stream

fn handle_connection(mut stream: TcpStream) {
    let mut parser = RESPParser::new(stream);
    parser.parse_next().expect("Can not parse next");
}
