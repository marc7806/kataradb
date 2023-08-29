use std::io::{BufRead, Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpListener, TcpStream};

use crate::resp::DataType::SimpleString;
use crate::resp::RESPParser;

pub mod resp;

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
    let mut parser = RESPParser::new(stream);

    while let Ok(data_type) = parser.decode_next() {
        println!("Got command: {:?}", data_type);
        // handle ping command
        match data_type {
            resp::DataType::Array(array) => {
                let cmd = &array[0];

                if cmd == &SimpleString(String::from("PING")) {
                    parser.write_to_stream(SimpleString(String::from("PONG")));
                    parser.flush_stream();
                } else {
                    println!("Got not supported command");
                }
            }
            _ => {
                println!("Got not supported command");
            }
        }
    }
}
