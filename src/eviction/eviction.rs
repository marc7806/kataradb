use crate::eviction::simple_eviction_policy::SimpleEvictionPolicy;
use crate::store::Store;

pub struct EvictionManager {
    pub max_items: u64,
    pub eviction_policy: Box<dyn EvictionPolicy>,
}

pub trait EvictionPolicy {
    fn evict(&self, store: &mut Store) -> Result<(), String>;
}

pub enum EvictionPolicyType {
    SIMPLE
}

impl EvictionPolicyType {
    pub fn get_eviction_policy(&self) -> Box<dyn EvictionPolicy> {
        match self {
            EvictionPolicyType::SIMPLE => Box::new(SimpleEvictionPolicy {}),
        }
    }
}

impl EvictionManager {
    pub fn new(max_items: u64, eviction_policy: Box<dyn EvictionPolicy>) -> EvictionManager {
        return EvictionManager {
            max_items,
            eviction_policy
        }
    }

    pub fn evict(&mut self, store: &mut Store) {
        println!("Evicting keys...");
        match self.eviction_policy.evict(store) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error while evicting keys: {}", err);
            }
        }
    }

    pub fn ready_for_evict(&self, store: &Store) -> bool {
        store.get_data().len() as u64 >= self.max_items
    }
}