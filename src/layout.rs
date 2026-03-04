use rdev::Key;
use std::sync::Arc;

#[derive(Debug)]
pub struct Layout {
    pub layer: Vec<Vec<Button>> 
}

#[derive(Debug)]
pub struct Button {
    pub width: u16,
    pub binds: Vec<(Arc<str>, Option<Key>)>
}

