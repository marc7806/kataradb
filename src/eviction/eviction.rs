use crate::eviction::all_keys_random_eviction_policy::AllKeysRandomEvictionPolicy;
use crate::eviction::simple_eviction_policy::SimpleEvictionPolicy;
use crate::store::Store;

pub struct EvictionManagerConfiguration {
    pub keys_limit: u64,
    pub eviction_ratio: f64,
}

pub struct EvictionManager {
    pub eviction_policy: Box<dyn EvictionPolicy>,
    pub config: EvictionManagerConfiguration,
}

pub trait EvictionPolicy {
    fn evict(&self, config: &EvictionManagerConfiguration, store: &mut Store) -> Result<(), String>;
}

pub enum EvictionPolicyType {
    Simple,
    AllKeysRandom
}

impl EvictionPolicyType {
    pub fn get_eviction_policy(&self) -> Box<dyn EvictionPolicy> {
        match self {
            EvictionPolicyType::Simple => Box::new(SimpleEvictionPolicy {}),
            EvictionPolicyType::AllKeysRandom => Box::new(AllKeysRandomEvictionPolicy {}),
        }
    }
}

impl EvictionManager {
    pub fn new(config: EvictionManagerConfiguration, eviction_policy: Box<dyn EvictionPolicy>) -> EvictionManager {
        return EvictionManager {
            config,
            eviction_policy
        }
    }

    pub fn evict(&mut self, store: &mut Store) {
        println!("Evicting keys...");
        match self.eviction_policy.evict(&self.config, store) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error while evicting keys: {}", err);
            }
        }
    }

    pub fn ready_for_evict(&self, store: &Store) -> bool {
        store.get_data().len() as u64 >= self.config.keys_limit
    }
}
