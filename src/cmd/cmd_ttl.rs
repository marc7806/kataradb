use crate::cmd::handler::Command;
use crate::resp::DataType;
use crate::resp::DataType::Integer;
use crate::store::Store;

/// see https://redis.io/commands/ttl/
pub struct TTLCommand;

impl Command for TTLCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        let key = args[0].clone();

        return match store.get(key.as_str()) {
            Some(store_object) => {
                let now = chrono::Utc::now().timestamp_millis();
                let expires_at = store_object.expires_at;

                if expires_at == -1 {
                    Integer(-1)
                } else {
                    let ttl = expires_at - now;
                    let ttl_seconds = ttl / 1000;
                    Integer(ttl_seconds)
                }
            }
            None => {
                Integer(-2)
            }
        }
    }
}