use std::io::{BufRead, Read, Write};
use std::net::TcpStream;

use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, SimpleString};

pub mod resp;
pub mod io_multiplexer;
pub mod async_tcp_server;

// todo: add command handling for async server
fn main() {
    println!("Starting kataradb");
    async_tcp_server::setup_server();
}

fn handle_connection(stream: &mut TcpStream) {
    let mut parser = RESPParser::new();
    let data_type = parser.decode_next(stream).expect("Can not decode data type");

    println!("Got command: {:?}", data_type);

    // handle ping command
    handle_cmd(&mut parser, data_type, stream);
}

fn handle_cmd(parser: &mut RESPParser, data_type: DataType, stream: &mut TcpStream) {
    match data_type {
        resp::DataType::Array(array) => {
            let cmd = &array[0];

            if cmd == &BulkString(String::from("COMMAND")) {} else if cmd == &BulkString(String::from("PING")) {
                parser.write_to_stream(stream, SimpleString(String::from("PONG")));
                parser.flush_stream(stream);

                while let Ok(data_type) = parser.decode_next(stream) {
                    handle_cmd(parser, data_type, stream);
                }
            } else {}
        }
        _ => {
            println!("Got not supported command");
        }
    }
}
