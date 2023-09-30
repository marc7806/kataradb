use crate::cmd::handler::Command;
use crate::resp::DataType;
use crate::resp::DataType::Integer;
use crate::store::Store;

/// see https://redis.io/commands/ttl/
pub struct TTLCommand;

impl Command for TTLCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        let key = args[0].clone();

        return match store.get_expiry(key.as_str()) {
            Some(expires_at) => {
                let now = chrono::Utc::now().timestamp_millis();
                let ttl = expires_at - now;
                let ttl_seconds = ttl / 1000;
                Integer(ttl_seconds)
            }
            None => {
                Integer(-1)
            }
        }
    }
}
