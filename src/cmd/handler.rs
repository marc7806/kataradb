use std::collections::HashMap;
use std::net::TcpStream;

use crate::cmd::cmd_del::DelCommand;
use crate::cmd::cmd_expire::ExpireCommand;
use crate::cmd::cmd_get::GetCommand;
use crate::cmd::cmd_ping::PingCommand;
use crate::cmd::cmd_set::SetCommand;
use crate::cmd::cmd_ttl::TTLCommand;
use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, SimpleString};
use crate::store::Store;

pub trait Command {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType;
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
        commands.insert(BulkString(String::from("EXPIRE")), Box::new(ExpireCommand));

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
                    Some(command) => {
                        let mut args = self.extract_args(&data);
                        match args {
                            Ok(mut result) => {
                                let command_result = command.execute(&mut result, store);
                                self.parser.write_to_stream(stream, command_result);
                                self.parser.flush_stream(stream);
                            }
                            Err(err) => {
                                self.parser.write_to_stream(stream, DataType::Error(err));
                                self.parser.flush_stream(stream);
                            }
                        }
                    }
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

    fn extract_args(&self, data: &Vec<DataType>) -> Result<Vec<String>, String> {
        let mut result = Vec::new();

        // skip first element, because it is the command
        for i in 1..data.len() {
            match &data[i] {
                BulkString(value) => {
                    result.push(value.clone());
                }
                _ => {
                    return Err(String::from("Wrong argument type"));
                }
            }
        }

        Ok(result)
    }
}