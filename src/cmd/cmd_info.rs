use crate::cmd::command::Command;
use crate::resp::DataType;
use crate::stats::KEYSPACE_STATISTICS;
use crate::store::Store;

/// see https://redis.io/commands/info/

pub struct InfoCommand;

impl Command for InfoCommand {
    fn execute(&self, _: &mut Vec<String>, _: &mut Store) -> DataType {
        let mut response = String::new();
        response.push_str("# Keyspace\r\n");

        for (keyspace_id, keyspace_stats) in KEYSPACE_STATISTICS.lock().unwrap().iter().enumerate() {
            response.push_str(&format!("db{}:keys={}\r\n", keyspace_id, keyspace_stats.number_of_keys));
        }

        DataType::BulkString(response)
    }
}
