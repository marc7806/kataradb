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

fn handle_connection(stream: TcpStream) {
    let mut parser = RESPParser::new(stream);
    let data_type = parser.decode_next().expect("Can not decode data type");

    println!("Got command: {:?}", data_type);

    // handle ping command
    handle_cmd(&mut parser, data_type);
}

fn handle_cmd(parser: &mut RESPParser, data_type: DataType) {
    match data_type {
        resp::DataType::Array(array) => {
            let cmd = &array[0];

            if cmd == &BulkString(String::from("COMMAND")) {} else if cmd == &BulkString(String::from("PING")) {
                parser.write_to_stream(SimpleString(String::from("PONG")));
                parser.flush_stream();

                while let Ok(data_type) = parser.decode_next() {
                    handle_cmd(parser, data_type);
                }
            } else {}
        }
        _ => {
            println!("Got not supported command");
        }
    }
}
