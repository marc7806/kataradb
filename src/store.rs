use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::eviction::eviction::EvictionManager;
use crate::eviction::eviction::EvictionPolicyType::SIMPLE;

#[derive(Debug, PartialEq, Clone)]
pub struct StoreObject {
    pub data: String,
    // stores the expiration in unix epoch milliseconds
    pub expires_at: i64,
}

impl StoreObject {
    pub fn get_data(&self) -> String {
        self.data.clone()
    }
}

pub struct Store {
    data: HashMap<String, StoreObject>,
    eviction_manager: Option<EvictionManager>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: HashMap::new(),
            eviction_manager: Some(EvictionManager::new(5, SIMPLE.get_eviction_policy())),
        }
    }

    pub fn put(&mut self, key: &str, value: String, expiration_duration_ms: i64) {
        // check for eviction
        let mut eviction_manager = self.eviction_manager.take().expect("EvictionManager is None");
        if eviction_manager.ready_for_evict(self) {
            eviction_manager.evict(self);
        }
        self.eviction_manager = Some(eviction_manager);
        //

        let expires_at =
            if expiration_duration_ms > 0 {
                let now = chrono::Utc::now();
                let duration = chrono::Duration::milliseconds(expiration_duration_ms);
                let expires_at = now + duration;
                expires_at.timestamp_millis()
            } else {
                -1
            };

        let store_object = StoreObject {
            data: value,
            expires_at,
        };

        self.data.insert(String::from(key), store_object);
    }

    pub fn remove(&mut self, key: &str) -> Option<StoreObject> {
        return self.data.remove(key);
    }

    pub fn get(&mut self, key: &str) -> Option<StoreObject> {
        match self.data.entry(key.to_string()) {
            Entry::Occupied(entry) => {
                // Check whether store_object is expired
                let store_object = entry.get();
                let now = chrono::Utc::now().timestamp_millis();
                if store_object.expires_at != -1 && store_object.expires_at < now {
                    entry.remove();
                    return None;
                }

                Some(store_object.clone())
            }
            Entry::Vacant(_) => {
                return None;
            }
        }
    }

    pub fn get_data(&self) -> &HashMap<String, StoreObject> {
        &self.data
    }
}

#[test]
fn test_store_put_get() {
    // given
    let mut store = Store::new();

    // when
    store.put("key", String::from("value"), -1);
    store.put("key2", String::from("value2"), 1000);
    store.put("key3", String::from("value3"), 2000);
    store.put("key4", String::from("value4"), 3000);

    // then
    assert_eq!(store.get("key").unwrap().data, String::from("value"));
    assert_eq!(store.get("key").unwrap().expires_at, -1);

    assert_eq!(store.get("key2").unwrap().data, String::from("value2"));
    assert_eq!(store.get("key2").unwrap().expires_at, chrono::Utc::now().timestamp_millis() + 1000);

    assert_eq!(store.get("key3").unwrap().data, String::from("value3"));
    assert_eq!(store.get("key3").unwrap().expires_at, chrono::Utc::now().timestamp_millis() + 2000);

    assert_eq!(store.get("key4").unwrap().data, String::from("value4"));
    assert_eq!(store.get("key4").unwrap().expires_at, chrono::Utc::now().timestamp_millis() + 3000);
}

#[test]
fn test_store_remove() {
    // given
    let mut store = Store::new();
    store.put("key", String::from("value"), -1);
    store.put("key2", String::from("value2"), 1000);
    store.put("key3", String::from("value3"), 2000);

    // then
    let removed_key = store.remove("key");
    let removed_key_2 = store.remove("key2");
    let removed_key_3 = store.remove("key3");
    let not_existing_key = store.remove("notExistingKey");

    // when
    assert_eq!(removed_key, Some(StoreObject {
        data: String::from("value"),
        expires_at: -1,
    }));
    assert_eq!(removed_key_2, Some(StoreObject {
        data: String::from("value2"),
        expires_at: chrono::Utc::now().timestamp_millis() + 1000,
    }));
    assert_eq!(removed_key_3, Some(StoreObject {
        data: String::from("value3"),
        expires_at: chrono::Utc::now().timestamp_millis() + 2000,
    }));
    assert_eq!(not_existing_key, None);
}
