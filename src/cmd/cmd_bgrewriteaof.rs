use std::fs::File;
use std::io::Write;

use crate::cmd::handler::Command;
use crate::resp::{DataType, RESPParser};
use crate::store::{Store, store_object_to_datatype};

/// see: https://redis.io/commands/bgrewriteaof
pub struct BgRewriteAofCommand;

const AOF_FILE_NAME: &str = "kataradb.aof";

// todo: run aof rewrite in background process instead of doing it synchronously
impl Command for BgRewriteAofCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        println!("Rewriting AOF file...");

        let mut parser = RESPParser::new();
        let mut aof_file = File::create(AOF_FILE_NAME).expect("Can not create AOF file");

        for (key, value) in store.get_data().iter() {
            let command = DataType::Array(vec![
                DataType::BulkString(String::from("SET")),
                DataType::BulkString(key.to_string()),
                store_object_to_datatype(value),
            ]);
            let encoded = parser.encode(command);
            aof_file.write_all(encoded.as_bytes()).expect("Can not write to AOF file");
        }

        return DataType::SimpleString("OK".to_string());
    }
}