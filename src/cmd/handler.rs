use std::collections::HashMap;
use DataType::Error;
use crate::client::ClientConnection;

use crate::cmd::command::{Command, get_commands, is_simple_command, SimpleCommand};
use crate::cmd::transaction::{is_transaction_command, TransactionCommand};
use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, SimpleString};
use crate::store::Store;

const OK: &str = "OK";
const NOT_SUPPORTED_COMMAND: &str = "Not supported command";
const WRONG_ARGUMENT_TYPE: &str = "Wrong argument type";
const QUEUED: &str = "QUEUED";

pub struct CommandHandler {
    commands: HashMap<SimpleCommand, Box<dyn Command>>,
    parser: RESPParser,
}

impl CommandHandler {
    pub fn new() -> Self {
        let parser = RESPParser::new();
        let commands = get_commands();

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
            if cmd_request.as_array().is_none() {
                results.push(Error(NOT_SUPPORTED_COMMAND.to_string()));
                continue;
            }

            let request = cmd_request.as_array().unwrap();
            let command = &request[0];

            if let Some(transaction_cmd) = is_transaction_command(command) {
                let result = self.execute_transaction_command(transaction_cmd, connection, store);
                results.push(result);
            } else if connection.is_transaction_active {
                connection.cmd_queue.push(cmd_request);
                results.push(SimpleString(QUEUED.to_string()));
            } else {
                let result = self.handle_simple_command_request(cmd_request, store);
                results.push(result);
            }
        }

        self.parser.write_to_stream(&mut connection.stream, results);
        self.parser.flush_stream(&mut connection.stream);
    }

    pub fn handle_simple_command_request(&mut self, cmd_request: DataType, store: &mut Store) -> DataType {
        if cmd_request.as_array().is_none() {
            return Error(NOT_SUPPORTED_COMMAND.to_string());
        }

        let request = cmd_request.as_array().unwrap();
        if request.len() < 1 {
            return Error(NOT_SUPPORTED_COMMAND.to_string());
        }

        if is_simple_command(&request[0]).is_none() {
            return Error(NOT_SUPPORTED_COMMAND.to_string());
        }

        let command = is_simple_command(&request[0]).unwrap();

        let args = self.extract_args(&request);
        if args.is_none() {
            return Error(WRONG_ARGUMENT_TYPE.to_string());
        }

        return self.execute_simple_command(&command, &mut args.unwrap(), store);
    }

    pub fn execute_simple_command(&mut self, command: &SimpleCommand, args: &mut Vec<String>, store: &mut Store) -> DataType {
        return match self.commands.get(&command) {
            Some(command) => {
                command.execute(args, store)
            }
            None => {
                SimpleString(OK.to_string())
            }
        }
    }

    fn execute_transaction_command(&mut self, cmd: TransactionCommand, client_connection: &mut ClientConnection, store: &mut Store) -> DataType {
        match cmd {
            TransactionCommand::MULTI => {
                client_connection.is_transaction_active = true;
                println!("Transaction started");
                SimpleString(OK.to_string())
            }
            TransactionCommand::EXEC => {
                let mut results = Vec::new();
                for cmd in client_connection.cmd_queue.drain(..) {
                    let result = self.handle_simple_command_request(cmd, store);
                    results.push(result);
                }

                client_connection.is_transaction_active = false;
                DataType::Array(results)
            }
            TransactionCommand::DISCARD => {
                println!("Transaction discarded");
                client_connection.cmd_queue.clear();
                client_connection.is_transaction_active = false;
                SimpleString(OK.to_string())
            }
        }
    }

    fn extract_args(&self, data: &Vec<DataType>) -> Option<Vec<String>> {
        let mut result = Vec::new();

        // skip first element, because it is the command
        for i in 1..data.len() {
            match &data[i] {
                BulkString(value) => {
                    result.push(value.clone());
                }
                _ => {
                    return None;
                }
            }
        }

        Some(result)
    }
}
