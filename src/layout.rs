use rdev::Key;
use std::sync::Arc;

#[derive(Debug)]
pub struct Layout {
    pub layer: Vec<Vec<Button>> 
}

#[derive(Debug)]
pub struct Button {
    pub name: Arc<str>,
    pub width: u16,
    pub rdev_key: Option<Key>,
}

