use crate::store::Store;

/// see https://redis.io/docs/reference/eviction/

pub struct EvictionManagerConfiguration {
    pub keys_limit: u64,
    pub eviction_ratio: f64,
}

impl EvictionManagerConfiguration {
    pub fn get_keys_to_remove(&self) -> u64 {
        return (self.keys_limit as f64 * self.eviction_ratio) as u64;
    }
}

pub struct EvictionManager {
    pub strategy: Box<dyn EvictionStrategy>,
    pub config: EvictionManagerConfiguration,
}

pub trait EvictionStrategy {
    fn evict(&mut self, config: &EvictionManagerConfiguration, store: &mut Store) -> Result<(), String>;
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
