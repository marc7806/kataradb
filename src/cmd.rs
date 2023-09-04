use std::collections::HashMap;
use std::net::TcpStream;

use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, SimpleString};
use crate::store::Store;

trait Command {
    fn execute(&self, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store);
}

struct PingCommand;
impl Command for PingCommand {
    fn execute(&self, parser: &mut RESPParser, stream: &mut TcpStream, _: &mut Store) {
        parser.write_to_stream(stream, SimpleString(String::from("PONG")));
        parser.flush_stream(stream);
    }
}

struct SetCommand;
impl Command for SetCommand {
    fn execute(&self, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        parser.write_to_stream(stream, SimpleString(String::from("OK")));
        parser.flush_stream(stream);
    }
}

pub struct CommandHandler {
    commands: HashMap<DataType, Box<dyn Command>>,
    parser: RESPParser,
}

impl CommandHandler {
    pub fn new() -> Self {
        let mut parser = RESPParser::new();

        let mut commands: HashMap<DataType, Box<dyn Command>> = HashMap::new();
        commands.insert(BulkString(String::from("PING")), Box::new(PingCommand));
        commands.insert(BulkString(String::from("SET")), Box::new(SetCommand));

        CommandHandler {
            commands,
            parser
        }
    }

    pub fn handle(&mut self, stream: &mut TcpStream, store: &mut Store) {
        let request = self.parser.decode_next(stream).expect("Can not decode data type");
        println!("Received command: {:?}", request);

        match request {
            DataType::Array(array) => {
                let cmd = &array[0];

                match self.commands.get(cmd) {
                    Some(command) => command.execute(&mut self.parser, stream, store),
                    None => {
                        self.parser.write_to_stream(stream, SimpleString(String::from("OK")));
                        self.parser.flush_stream(stream);
                    }
                }

            }
            _ => {
                println!("Got not supported command");
            }
        }

    }
}