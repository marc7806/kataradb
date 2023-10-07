use std::collections::HashMap;
use std::net::TcpStream;

use crate::cmd::cmd_bgrewriteaof::BgRewriteAofCommand;
use crate::cmd::cmd_del::DelCommand;
use crate::cmd::cmd_expire::ExpireCommand;
use crate::cmd::cmd_get::GetCommand;
use crate::cmd::cmd_incr::IncrCommand;
use crate::cmd::cmd_info::InfoCommand;
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
        let parser = RESPParser::new();

        let mut commands: HashMap<DataType, Box<dyn Command>> = HashMap::new();
        commands.insert(BulkString(String::from("PING")), Box::new(PingCommand));
        commands.insert(BulkString(String::from("SET")), Box::new(SetCommand));
        commands.insert(BulkString(String::from("GET")), Box::new(GetCommand));
        commands.insert(BulkString(String::from("TTL")), Box::new(TTLCommand));
        commands.insert(BulkString(String::from("DEL")), Box::new(DelCommand));
        commands.insert(BulkString(String::from("EXPIRE")), Box::new(ExpireCommand));
        commands.insert(BulkString(String::from("BGREWRITEAOF")), Box::new(BgRewriteAofCommand));
        commands.insert(BulkString(String::from("INCR")), Box::new(IncrCommand));
        commands.insert(BulkString(String::from("INFO")), Box::new(InfoCommand));

        CommandHandler {
            commands,
            parser,
        }
    }

    /// Handle commands in a pipeline
    pub fn handle_bulk(&mut self, stream: &mut TcpStream, store: &mut Store) {
        let mut cmd_requests = self.parser.decode_next_bulk(stream).expect("Can not decode data type");
        println!("Received commands: {:?}", cmd_requests);

        let mut results = Vec::new();
        for cmd_request in cmd_requests.drain(..) {
            let result = self.execute_cmd(store, cmd_request);
            results.push(result);
        }

        self.parser.write_to_stream(stream, results);
        self.parser.flush_stream(stream);
    }

    pub fn execute_cmd(&mut self, store: &mut Store, request: DataType) -> DataType {
        return match request {
            DataType::Array(data) => {
                let requested_cmd = &data[0];

                match self.commands.get(requested_cmd) {
                    Some(command) => {
                        let result_args = self.extract_args(&data);
                        match result_args {
                            Ok(mut args) => {
                                command.execute(&mut args, store)
                            }
                            Err(err) => {
                                DataType::Error(err)
                            }
                        }
                    }
                    None => {
                        SimpleString(String::from("OK"))
                    }
                }
            }
            _ => {
                println!("Got not supported command");
                DataType::Error(String::from("Not supported command"))
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
