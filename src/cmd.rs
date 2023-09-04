use std::collections::HashMap;
use std::net::TcpStream;

use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, SimpleString};

trait Command {
    fn execute(&self, parser: &mut RESPParser, stream: &mut TcpStream);
}

struct PingCommand;
impl Command for PingCommand {
    fn execute(&self, parser: &mut RESPParser, stream: &mut TcpStream) {
        parser.write_to_stream(stream, SimpleString(String::from("PONG")));
        parser.flush_stream(stream);
    }
}

struct SetCommand;
impl Command for SetCommand {
    fn execute(&self, parser: &mut RESPParser, stream: &mut TcpStream) {
        parser.write_to_stream(stream, SimpleString(String::from("OK")));
        parser.flush_stream(stream);
    }
}

pub struct CommandHandler {
    commands: HashMap<DataType, Box<dyn Command>>,
}

impl CommandHandler {
    pub fn new() -> Self {
        let mut commands: HashMap<DataType, Box<dyn Command>> = HashMap::new();
        commands.insert(BulkString(String::from("PING")), Box::new(PingCommand));
        commands.insert(BulkString(String::from("SET")), Box::new(SetCommand));

        CommandHandler {
            commands,
        }
    }

    pub fn handle(&self, parser: &mut RESPParser, stream: &mut TcpStream, request: DataType) {
        println!("Got command: {:?}", request);

        match request {
            DataType::Array(array) => {
                let cmd = &array[0];

                match self.commands.get(cmd) {
                    Some(command) => command.execute(parser, stream),
                    None => {
                        parser.write_to_stream(stream, SimpleString(String::from("OK")));
                        parser.flush_stream(stream);
                    }
                }

            }
            _ => {
                println!("Got not supported command");
            }
        }

    }
}