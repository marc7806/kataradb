use crate::eviction::eviction::{EvictionManagerConfiguration, EvictionStrategy};
use crate::store::Store;

pub struct SimpleEvictionStrategy {}

impl EvictionStrategy for SimpleEvictionStrategy {
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
