use crate::eviction::eviction::{EvictionManagerConfiguration, EvictionStrategy};
use crate::store::{Store, StoreObject};

/// Approximate LRU eviction strategy
/// N new keys get selected randomly and checked against the pool.
/// If keys are older than we add/replace them to pool and so the worst candidates are taken out of the pool.
/// Then we evict the oldest keys from the pool.

#[derive(PartialEq)]
pub struct EvictionPoolItem {
    key: String,
    obj_ptr: *const StoreObject,
}

pub struct AllKeysLRUEvictionStrategy {
    pool: Vec<EvictionPoolItem>,
    sample_size: usize,
}

impl AllKeysLRUEvictionStrategy {
    pub fn new() -> Self {
        AllKeysLRUEvictionStrategy {
            pool: Vec::with_capacity(15),
            sample_size: 5,
        }
    }

    fn populate_pool(&mut self, store: &mut Store) {
        for (key, obj) in store.get_data().iter().take(self.sample_size) {
            // sort array ascending by last_accessed_at
            // TODO: is not very efficient to sort everytime
            self.sort_by_last_accessed_at();

            let obj_ptr = obj as *const StoreObject;

            let item = EvictionPoolItem {
                key: key.clone(),
                obj_ptr
            };

            if self.pool.contains(&item) {
                continue;
            }

            if self.pool.len() < self.pool.capacity() {
                self.pool.push(item);
            } else {
                // check whether element is older than oldest element in array
                // as we sort the array by last_accessed_at we can just check the last element
                let oldest_item = self.pool.first().unwrap();
                let oldest_item_obj = unsafe { &*oldest_item.obj_ptr };

                if obj.last_accessed_at < oldest_item_obj.last_accessed_at {
                    self.pool[0] = item;
                }
            }
        }
    }

    fn sort_by_last_accessed_at(&mut self) {
        self.pool.sort_by(|a, b| {
            let a_obj = unsafe { &*a.obj_ptr };
            let b_obj = unsafe { &*b.obj_ptr };

            a_obj.last_accessed_at.cmp(&b_obj.last_accessed_at)
        });
    }
}

impl EvictionStrategy for AllKeysLRUEvictionStrategy {
    fn evict(&mut self, config: &EvictionManagerConfiguration, store: &mut Store) -> Result<(), String> {
        self.populate_pool(store);

        // it is possible that the keys we evict are already removed by the user
        self.pool.drain(0..config.get_keys_to_remove() as usize).into_iter().for_each(|item| {
            println!("Evicting key: {}", item.key);
            store.remove(item.key.as_str());
        });

        Ok(())
    }
}
