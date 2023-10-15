use crate::cmd::command::Command;
use crate::resp::DataType;
use crate::resp::DataType::Integer;
use crate::store::Store;

/// see: https://redis.io/commands/del/
pub struct DelCommand;

impl Command for DelCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        let mut deleted = 0;

        for key_to_delete in args.iter() {
            let result = store.remove(key_to_delete);
            if result.is_some() {
                deleted += 1;
            }
        }

        return Integer(deleted);
    }
}
