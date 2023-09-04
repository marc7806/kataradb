use std::any::Any;
use std::collections::HashMap;

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

    pub fn get<T: Any>(&self, key: &str) -> Option<&T> {
        self.data
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
    }
}