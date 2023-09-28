use crate::eviction::eviction::{EvictionManagerConfiguration, EvictionPolicy};
use crate::store::Store;

pub struct SimpleEvictionPolicy {}

impl EvictionPolicy for SimpleEvictionPolicy {
    fn evict(&self, _: &EvictionManagerConfiguration, store: &mut Store) -> Result<(), String> {
        let mut key_to_remove = String::new();
        for (key, _) in store.get_data().iter() {
            key_to_remove = key.clone();
            break;
        }
        println!("Evicted key: {}", key_to_remove);
        store.remove(&key_to_remove);
        
        Ok(())
    }
}
