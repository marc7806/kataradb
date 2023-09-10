use std::time::{SystemTime, UNIX_EPOCH};

use crate::store::Store;

/// Implement redis active expiration
/// https://redis.io/commands/expire
/// Define cronInterval in which you randomly select 20 keys with expiration set
/// if they have expiration set and expiration is in the past, delete the key
/// While doing this increase a counter and check whether we have deleted more than 25% of the keys
/// if so, repeat the process

pub struct ActiveExpirationManager {
    cron_interval_ms: u64,
    last_run: u64,
    deleted_keys: u64,
    total_keys: u64,
}

impl ActiveExpirationManager {
    pub fn new(cron_interval: u64) -> Self {
        Self {
            cron_interval_ms: cron_interval,
            last_run: 0,
            deleted_keys: 0,
            total_keys: 0,
        }
    }

    pub fn run_loop(&mut self, store: &mut Store) {
        loop {
            if self.run(store) {
                println!("Expiration Manager: deleted {} keys out of {} total keys", self.deleted_keys, self.total_keys);
            } else {
                break;
            }
        }
    }

    fn run(&mut self, store: &mut Store) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        if (now - self.last_run) < self.cron_interval_ms {
            // Cron interval not reached yet
            return false;
        }

        let mut num_keys_with_expiration = 0;
        let mut num_deleted_keys = 0;
        let mut keys_to_delete = Vec::new();
        let max_keys_with_expiration_to_visit = 20;

        // iterate in a random order over the data of the store until we saw 20 keys with expiration and delete expired keys
        for (key, store_object) in store.get_data().iter() {
            if store_object.expires_at != -1 {
                num_keys_with_expiration += 1;
                if store_object.expires_at < now as i64 {
                    num_deleted_keys += 1;
                    keys_to_delete.push(key.clone());
                }
            }

            if num_keys_with_expiration >= max_keys_with_expiration_to_visit {
                break;
            }
        }

        for key in keys_to_delete {
            store.remove(&key);
        }

        self.total_keys = store.get_data().len() as u64;
        self.last_run = now;
        self.deleted_keys = num_deleted_keys;

        let exp_keys_ratio = num_deleted_keys as f64 / num_keys_with_expiration as f64;
        return exp_keys_ratio > 0.25;
    }
}