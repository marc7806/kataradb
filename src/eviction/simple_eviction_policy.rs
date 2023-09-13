use crate::eviction::eviction::EvictionPolicy;
use crate::store::Store;

pub struct SimpleEvictionPolicy {}

impl EvictionPolicy for SimpleEvictionPolicy {
    fn evict(&self, store: &mut Store) -> Result<(), String> {
        let mut key_to_remove = String::new();
        for (key, _) in store.get_data().iter() {
            key_to_remove = key.clone();
            break;
        }
        println!("Evicted key: {}", key_to_remove);
        store.remove(&key_to_remove);
        return Ok(());
    }
}