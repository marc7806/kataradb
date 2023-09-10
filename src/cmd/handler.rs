use std::collections::HashMap;
use std::net::TcpStream;

use crate::cmd::cmd_del::DelCommand;
use crate::cmd::cmd_get::GetCommand;
use crate::cmd::cmd_ping::PingCommand;
use crate::cmd::cmd_set::SetCommand;
use crate::cmd::cmd_ttl::TTLCommand;
use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, SimpleString};
use crate::store::Store;

pub trait Command {
    fn execute(&self, data: &mut Vec<DataType>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store);
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
        commands.insert(BulkString(String::from("GET")), Box::new(GetCommand));
        commands.insert(BulkString(String::from("TTL")), Box::new(TTLCommand));
        commands.insert(BulkString(String::from("DEL")), Box::new(DelCommand));

        CommandHandler {
            commands,
            parser,
        }
    }

    pub fn handle(&mut self, stream: &mut TcpStream, store: &mut Store) {
        let request = self.parser.decode_next(stream).expect("Can not decode data type");
        println!("Received command: {:?}", request);

        match request {
            DataType::Array(mut data) => {
                let cmd = &data[0];

                match self.commands.get(cmd) {
                    Some(command) => command.execute(&mut data, &mut self.parser, stream, store),
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