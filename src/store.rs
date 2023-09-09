use std::any::Any;
use std::collections::HashMap;

use crate::resp::DataType;

pub struct Store {
    // box because we want to store data of any type on heap
    data: HashMap<String, Box<dyn Any>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: HashMap::new(),
        }
    }

    pub fn put<T: Any>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Box::new(value));
    }

    pub fn get<T: Any>(&self, key: &str) -> Result<&T, ()> {
        match self.data.get(key) {
            Some(value) => value.downcast_ref::<T>().ok_or(()),
            None => Err(()),
        }
    }
}

#[test]
fn test_store() {
    // given
    let mut store = Store::new();

    // when
    store.put("key", 1);
    store.put("key2", "value");
    store.put("key3", DataType::BulkString(String::from("bulk-string-example")));

    // then
    assert_eq!(store.get::<i32>("key"), Ok(&1));
    assert_eq!(store.get::<&str>("key2"), Ok(&"value"));
    assert_eq!(store.get::<DataType>("key3"), Ok(&DataType::BulkString(String::from("bulk-string-example"))));
}
