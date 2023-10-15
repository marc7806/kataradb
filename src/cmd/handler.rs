use std::collections::HashMap;
use crate::client::ClientConnection;

use crate::cmd::cmd_bgrewriteaof::BgRewriteAofCommand;
use crate::cmd::cmd_del::DelCommand;
use crate::cmd::cmd_expire::ExpireCommand;
use crate::cmd::cmd_get::GetCommand;
use crate::cmd::cmd_incr::IncrCommand;
use crate::cmd::cmd_info::InfoCommand;
use crate::cmd::cmd_ping::PingCommand;
use crate::cmd::cmd_set::SetCommand;
use crate::cmd::cmd_ttl::TTLCommand;
use crate::cmd::transaction::{is_transaction_command, TransactionCommand};
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
    pub fn handle_bulk(&mut self, connection: &mut ClientConnection, store: &mut Store) {
        let mut cmd_requests = self.parser.decode_next_bulk(&mut connection.stream).expect("Can not decode data type");
        println!("Received commands: {:?}", cmd_requests);

        let mut results = Vec::new();
        for cmd_request in cmd_requests.drain(..) {
            if !cmd_request.is_array() {
                results.push(DataType::Error(String::from("Not supported command")));
                continue;
            }

            let command = &cmd_request.as_array()[0];

            if let Some(transaction_cmd) = is_transaction_command(&command) {
                let result = self.execute_transaction_command(transaction_cmd, connection, store);
                results.push(result);
            } else if connection.is_transaction_active {
                connection.cmd_queue.push(cmd_request.clone());
                results.push(SimpleString(String::from("QUEUED")));
            } else {
                let result = self.execute_cmd(store, cmd_request);
                results.push(result);
            }
        }

        self.parser.write_to_stream(&mut connection.stream, results);
        self.parser.flush_stream(&mut connection.stream);
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
        };
    }

    fn execute_transaction_command(&mut self, cmd: TransactionCommand, client_connection: &mut ClientConnection, store: &mut Store) -> DataType {
        match cmd {
            TransactionCommand::MULTI => {
                client_connection.is_transaction_active = true;
                println!("Transaction started");
                SimpleString(String::from("OK"))
            }
            TransactionCommand::EXEC => {
                let mut results = Vec::new();
                for cmd in client_connection.cmd_queue.drain(..) {
                    let result = self.execute_cmd(store, cmd);
                    results.push(result);
                }

                client_connection.is_transaction_active = false;
                DataType::Array(results)
            }
            TransactionCommand::DISCARD => {
                println!("Transaction discarded");
                client_connection.cmd_queue.clear();
                client_connection.is_transaction_active = false;
                SimpleString(String::from("OK"))
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
