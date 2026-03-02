use ratatui::{prelude::*, widgets::*};
use ratatui::layout::{Layout as TuiLayout, Constraint, Direction, Alignment};
use rdev::Key;
use std::{
    collections::HashSet,

};

use crate::layout::Layout;

pub fn render_ui(f: &mut Frame, pressed_keys: &HashSet<Key>, kps: usize, kbd_layout: &Layout) {
    let area = f.size();
    
    let color = Color::Rgb(176, 176, 176);
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" Terminal Virtual Keyboard")
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(color));
    
    let inner_area = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let chunks = TuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner_area);

    f.render_widget(
        Paragraph::new(format!("KPS: {} ", kps))
            .alignment(Alignment::Right)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        chunks[0]
    );

    let row_areas = TuiLayout::default()
        .direction(Direction::Vertical)
        .constraints(kbd_layout.layer.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>())
        .split(chunks[1]);

    for (r_idx, row) in kbd_layout.layer.iter().enumerate() {
        let key_constraints: Vec<Constraint> = row.iter().map(|k| Constraint::Length(k.width)).collect();
        let key_areas = TuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints(key_constraints)
            .split(row_areas[r_idx]);

        for (k_idx, key_def) in row.iter().enumerate() {
            let is_pressed = key_def.rdev_key.map_or(false, |k| pressed_keys.contains(&k));
            
            let style = if is_pressed {
                Style::default().bg(color).fg(Color::Black)
            } else {
                Style::default().fg(Color::Gray)
            };

            let key_label = Paragraph::new(key_def.name.as_ref())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Plain)
                        .style(style)
                );

            f.render_widget(key_label, key_areas[k_idx]);
        }
    }
}
