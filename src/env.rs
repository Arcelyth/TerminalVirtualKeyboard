use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Value {
    Number(u16), 
    RGB(u8, u8, u8)
}

#[derive(Debug)]
pub struct Env {
    values: HashMap<Arc<str>, Value>    
}

impl Env {
    pub fn new() -> Self {
        let mut vs = Self {
            values: HashMap::new()
        };

        // inner variables
        vs.insert("border_color", Value::RGB(176, 176, 176));
        vs
    }

    pub fn insert(&mut self, name: &str, value: Value) {
        self.values.insert(name.into(), value);
    }

    pub fn get(&self, k: &str) -> Option<&Value> {
        self.values.get(k.into())
    }
}
