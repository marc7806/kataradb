use std::collections::hash_map::Entry;
use std::collections::HashMap;
use crate::eviction::all_keys_random_eviction_strategy::AllKeysRandomEvictionStrategy;

use crate::eviction::eviction::{EvictionManager, EvictionManagerConfiguration};
use crate::object_type_encoding::{get_type, OBJ_ENCODING_EMBSTR, OBJ_ENCODING_INT, OBJ_ENCODING_RAW, OBJ_TYPE_STRING};
use crate::resp::DataType;
use crate::stats::update_keyspace_statistics;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum ObjectValue {
    String(String),
}

#[derive(Debug, Clone)]
pub struct StoreObject {
    // first 4 bits = type of object
    // last 4 bits = encoding of object
    // number-range = 0-15
    pub type_encoding: u8,
    // stores the actual value of the object, e.g. a pointer to a string
    pub value: Box<ObjectValue>,
    // as we have no native support for bitfields in Rust, we store the last accessed timestamp as 32bit instead of 24bit
    pub last_accessed_at: u32,
}

impl StoreObject {
    pub fn new(value: ObjectValue, type_encoding: u8) -> Self {
        StoreObject {
            value: Box::new(value),
            type_encoding,
            last_accessed_at: get_current_clock()
        }
    }

    pub fn get_value_clone(&self) -> ObjectValue {
        return self.value.as_ref().clone();
    }
}

pub struct Store {
    data: HashMap<String, StoreObject>,
    // stores the expiration of keys in unix epoch milliseconds
    expiration_data: HashMap<String, i64>,
    eviction_manager: Option<EvictionManager>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: HashMap::new(),
            expiration_data: HashMap::new(),
            eviction_manager: Some(EvictionManager::new(EvictionManagerConfiguration { keys_limit: 5, eviction_ratio: 0.4 }, Box::new(AllKeysRandomEvictionStrategy{}))),
        }
    }

    pub fn put(&mut self, key: &str, value: ObjectValue, expiration_duration_ms: i64, type_encoding: u8) {
        // check for eviction
        let mut eviction_manager = self.eviction_manager.take().expect("EvictionManager is None");
        if eviction_manager.ready_for_evict(self) {
            eviction_manager.evict(self);
        }
        self.eviction_manager = Some(eviction_manager);
        //

        let store_object = StoreObject::new(value, type_encoding);
        self.data.insert(String::from(key), store_object);

        if expiration_duration_ms > 0 {
            let now = chrono::Utc::now();
            let duration = chrono::Duration::milliseconds(expiration_duration_ms);
            let expires_at = now + duration;
            self.expiration_data.insert(String::from(key), expires_at.timestamp_millis());
        }

        update_keyspace_statistics(0, self.data.len() as u64);
    }

    pub fn remove(&mut self, key: &str) -> Option<StoreObject> {
        let removed_key = self.data.remove(key);
        self.expiration_data.remove(key);

        update_keyspace_statistics(0, self.data.len() as u64);

        removed_key
    }

    pub fn get(&mut self, key: &str) -> Option<StoreObject> {
        let has_expired = &self.has_expired(key);

        match self.data.entry(key.to_string()) {
            Entry::Occupied(mut entry) => {
                if *has_expired {
                    entry.remove();
                    return None;
                }

                entry.get_mut().last_accessed_at = get_current_clock();

                let store_object = entry.get();
                Some(store_object.clone())
            }
            Entry::Vacant(_) => {
                return None;
            }
        }
    }

    fn has_expired(&self, key: &str) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        let expires_at = self.expiration_data.get(key);

        if expires_at.is_none() {
            return false;
        }

       *expires_at.unwrap() <= now
    }

    pub fn get_data(&self) -> &HashMap<String, StoreObject> {
        &self.data
    }

    pub fn get_expiration_data(&self) -> &HashMap<String, i64> {
        &self.expiration_data
    }

    pub fn get_expiry(&self, key: &str) -> Option<i64> {
        self.expiration_data.get(key).cloned()
    }
}

pub fn store_object_to_datatype(value: &StoreObject) -> DataType {
    let obj_type = get_type(value.type_encoding);

    match obj_type {
        OBJ_TYPE_STRING => {
            return match value.value.as_ref() {
                ObjectValue::String(string) => {
                    DataType::BulkString(string.clone())
                }
            };
        }
        _ => {
            panic!("Unknown type");
        }
    }
}

fn get_current_clock() -> u32 {
    let now = chrono::Utc::now();
    let now = now.timestamp_millis();
    (now & 0xFFFFFFFF) as u32
}

#[test]
fn test_store_put_get() {
    // given
    let mut store = Store::new();

    // when
    store.put("key", ObjectValue::String("value".to_string()), -1, OBJ_TYPE_STRING | OBJ_ENCODING_RAW);
    store.put("key2", ObjectValue::String("123".to_string()), 1000, OBJ_TYPE_STRING | OBJ_ENCODING_INT);
    store.put("key4", ObjectValue::String(String::from("12345678901234567890123456789012345678901234567890test12345")), 2000, OBJ_TYPE_STRING | OBJ_ENCODING_EMBSTR);

    // then
    let key = store.get("key").expect("Key not found");
    assert_eq!(key.type_encoding, OBJ_TYPE_STRING | OBJ_ENCODING_RAW);
    assert_eq!(key.get_value_clone(), ObjectValue::String("value".to_string()));
    assert_eq!(store.get_expiry("key"), None);

    let key2 = store.get("key2").expect("Key not found");
    assert_eq!(key2.type_encoding, OBJ_TYPE_STRING | OBJ_ENCODING_INT);
    assert_eq!(key2.get_value_clone(), ObjectValue::String("123".to_string()));
    assert_eq!(store.get_expiry("key2").unwrap(), chrono::Utc::now().timestamp_millis() + 1000);

    let key4 = store.get("key4").expect("Key not found");
    assert_eq!(key4.type_encoding, OBJ_TYPE_STRING | OBJ_ENCODING_EMBSTR);
    assert_eq!(key4.get_value_clone(), ObjectValue::String(String::from("12345678901234567890123456789012345678901234567890test12345")));
    assert_eq!(store.get_expiry("key4").unwrap(), chrono::Utc::now().timestamp_millis() + 2000);
}

#[test]
fn test_store_remove() {
    // given
    let mut store = Store::new();
    store.put("key", ObjectValue::String("value".to_string()), -1, OBJ_TYPE_STRING | OBJ_ENCODING_RAW);
    store.put("key2", ObjectValue::String("123".to_string()), 1000, OBJ_TYPE_STRING | OBJ_ENCODING_INT);
    store.put("key4", ObjectValue::String(String::from("12345678901234567890123456789012345678901234567890test12345")), 2000, OBJ_TYPE_STRING | OBJ_ENCODING_EMBSTR);

    // then
    let removed_key = store.remove("key");
    let removed_key_2 = store.remove("key2");
    let removed_key_4 = store.remove("key4");
    let not_existing_key = store.remove("notExistingKey");

    // when
    assert_eq!(removed_key.expect("Key not found").value, Box::new(ObjectValue::String("value".to_string())));
    assert_eq!(removed_key_2.expect("Key not found").value, Box::new(ObjectValue::String("123".to_string())));
    assert_eq!(removed_key_4.expect("Key not found").value, Box::new(ObjectValue::String(String::from("12345678901234567890123456789012345678901234567890test12345"))));
    assert_eq!(not_existing_key.is_none(), true);
}
