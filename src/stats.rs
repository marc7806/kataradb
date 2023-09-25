use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

pub struct KeyspaceStatistics {
    pub number_of_keys: u64,
}

lazy_static! {
    pub static ref KEYSPACE_STATISTICS: Arc<Mutex<Vec<KeyspaceStatistics>>> = {
        let mut stats = Vec::new();
        stats.push(KeyspaceStatistics { number_of_keys: 0 });
        Arc::new(Mutex::new(stats))
    };
}

pub fn update_keyspace_statistics(keyspace_id: usize, number_of_keys: u64) {
    if let Ok(mut stats) = KEYSPACE_STATISTICS.lock() {
        if let Some(keyspace_stats) = stats.get_mut(keyspace_id) {
            keyspace_stats.number_of_keys = number_of_keys;
        }
    }
}
