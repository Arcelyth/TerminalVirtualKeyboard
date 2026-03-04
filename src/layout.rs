use ratatui::style::Color;
use rdev::Key;
use std::sync::Arc;

#[derive(Debug)]
pub struct Layout {
    pub layer: Vec<Vec<Button>>,
}

#[derive(Debug)]
pub struct Attr {
    pub width: u16,
    pub height: u16,
    pub border_color: Option<Color>,
    pub highlight: Option<Color>,
}

impl Attr {
    pub fn default(name: &str) -> Self {
        let width = match name.to_lowercase().as_str() {
            "space" => 20,
            _ => 4,
        };
        Self {
            width,
            height: 3,
            border_color: None,
            highlight: None,
        }
    }
}

#[derive(Debug)]
pub struct Button {
    pub attr: Attr,
    pub binds: Vec<(Arc<str>, Option<Key>)>,
}
