use crate::eviction::all_keys_random_eviction_strategy::AllKeysRandomEvictionStrategy;
use crate::eviction::simple_eviction_strategy::SimpleEvictionStrategy;
use crate::store::Store;

pub struct EvictionManagerConfiguration {
    pub keys_limit: u64,
    pub eviction_ratio: f64,
}

pub struct EvictionManager {
    pub strategy: Box<dyn EvictionStrategy>,
    pub config: EvictionManagerConfiguration,
}

pub trait EvictionStrategy {
    fn evict(&self, config: &EvictionManagerConfiguration, store: &mut Store) -> Result<(), String>;
}

impl EvictionManager {
    pub fn new(config: EvictionManagerConfiguration, strategy: Box<dyn EvictionStrategy>) -> EvictionManager {
        return EvictionManager {
            config,
            strategy
        }
    }

    pub fn evict(&mut self, store: &mut Store) {
        println!("Evicting keys...");
        match self.strategy.evict(&self.config, store) {
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
