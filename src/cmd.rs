use std::net::TcpStream;

use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, SimpleString};

pub fn handle_cmd(parser: &mut RESPParser, data_type: DataType, stream: &mut TcpStream) {
    println!("Got command: {:?}", data_type);
    match data_type {
        DataType::Array(array) => {
            let cmd = &array[0];

            if cmd == &BulkString(String::from("COMMAND")) {
                parser.write_to_stream(stream, SimpleString(String::from("OK")));
                parser.flush_stream(stream);
            } else if cmd == &BulkString(String::from("PING")) {
                parser.write_to_stream(stream, SimpleString(String::from("PONG")));
                parser.flush_stream(stream);
            } else {
            }
        }
        _ => {
            println!("Got not supported command");
        }
    }
}