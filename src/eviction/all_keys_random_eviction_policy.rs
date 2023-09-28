use crate::eviction::eviction::{EvictionManagerConfiguration, EvictionPolicy};
use crate::store::Store;

pub struct AllKeysRandomEvictionPolicy {}

impl EvictionPolicy for AllKeysRandomEvictionPolicy {
    fn evict(&self, config: &EvictionManagerConfiguration, store: &mut Store) -> Result<(), String> {
        let mut keys_to_remove = Vec::new();
        let num_keys_to_remove = (config.keys_limit as f64 * config.eviction_ratio) as usize;
        let mut i = 0;

        for (key, _) in store.get_data().iter() {
            if i >= num_keys_to_remove {
                break;
            }

            keys_to_remove.push(key.clone());
            i += 1;
        }

        println!("Evict {} keys", keys_to_remove.len());
        for key in keys_to_remove {
            store.remove(&key);
        }

        Ok(())
    }
}
