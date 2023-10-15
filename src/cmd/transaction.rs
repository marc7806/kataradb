use std::str::FromStr;
use crate::resp::DataType;
use crate::resp::DataType::BulkString;

#[derive(Debug, PartialEq)]
pub enum TransactionCommand {
    MULTI,
    EXEC,
    DISCARD,
}

impl FromStr for TransactionCommand {
    type Err = ();
    fn from_str(input: &str) -> Result<TransactionCommand, Self::Err> {
        match input {
            "MULTI" => Ok(TransactionCommand::MULTI),
            "EXEC" => Ok(TransactionCommand::EXEC),
            "DISCARD" => Ok(TransactionCommand::DISCARD),
            _ => Err(()),
        }
    }
}

pub fn is_transaction_command(cmd: &DataType) -> Option<TransactionCommand> {
    return match cmd {
        BulkString(value) => {
            value.parse::<TransactionCommand>().ok()
        }
        _ => {
            None
        }
    }
}
