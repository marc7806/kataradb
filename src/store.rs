use std::collections::HashMap;

use crate::resp::DataType;

pub struct StoreObject {
    pub data: DataType,
    // stores the expiration in unix epoch milliseconds
    pub expires_at: i64,
}

pub struct Store {
    // box because we want to store data of any type on heap
    data: HashMap<String, StoreObject>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: &str, value: DataType, expiration_duration_ms: i64) {
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

    pub fn get(&self, key: &str) -> Option<&StoreObject> {
        self.data.get(key)
    }
}

#[test]
fn test_store() {
    // given
    let mut store = Store::new();

    // when
    store.put("key", DataType::BulkString(String::from("value")), -1);
    store.put("key2", DataType::BulkString(String::from("value2")), 1000);
    store.put("key3", DataType::BulkString(String::from("value3")), 2000);
    store.put("key4", DataType::BulkString(String::from("value4")), 3000);

    // then
    assert_eq!(store.get("key").unwrap().data, DataType::BulkString(String::from("value")));
    assert_eq!(store.get("key").unwrap().expires_at, -1);

    assert_eq!(store.get("key2").unwrap().data, DataType::BulkString(String::from("value2")));
    assert_eq!(store.get("key2").unwrap().expires_at, chrono::Utc::now().timestamp_millis() + 1000);

    assert_eq!(store.get("key3").unwrap().data, DataType::BulkString(String::from("value3")));
    assert_eq!(store.get("key3").unwrap().expires_at, chrono::Utc::now().timestamp_millis() + 2000);

    assert_eq!(store.get("key4").unwrap().data, DataType::BulkString(String::from("value4")));
    assert_eq!(store.get("key4").unwrap().expires_at, chrono::Utc::now().timestamp_millis() + 3000);
}
