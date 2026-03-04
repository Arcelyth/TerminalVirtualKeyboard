use ratatui::{prelude::*, widgets::*};
use ratatui::layout::{Layout as TuiLayout, Constraint, Direction, Alignment};
use rdev::Key;
use std::{
    collections::HashSet,

};

use crate::layout::Layout;

pub fn render_ui(f: &mut Frame, pressed_keys: &HashSet<Key>, kps: usize, kbd_layout: &Layout) {
    let area = f.size();
    
    let default_border_color = Color::Rgb(176, 176, 176);
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" Terminal Virtual Keyboard ")
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(default_border_color));
    
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

        for (k_idx, button) in row.iter().enumerate() {
            let active_bind_idx = button.binds.iter().enumerate().rev()
                .find(|(_, (_, key))| key.map_or(false, |k| pressed_keys.contains(&k)))
                .map(|(i, _)| i);

            let (display_name, style) = match active_bind_idx {
                Some(idx) => {
                    let name = button.binds[idx].0.as_ref();
                    let layer_color = match idx {
                        0 => Color::Rgb(176, 176, 176),
                        1 => Color::Rgb(173, 173, 123), 
                        2 => Color::Rgb(123, 173, 144), 
                        _ => Color::Rgb(123, 159, 173), 
                    };
                    
                    (name, Style::default().bg(layer_color).fg(Color::Black).add_modifier(Modifier::BOLD))
                }
                None => {
                    let name = button.binds.get(0).map(|b| b.0.as_ref()).unwrap_or("");
                    (name, Style::default().fg(Color::Gray))
                }
            };

            let key_widget = Paragraph::new(display_name)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Plain)
                        .style(if active_bind_idx.is_some() { style } else { Style::default().fg(default_border_color) })
                );
            
            let final_widget = if active_bind_idx.is_some() {
                key_widget.style(style)
            } else {
                key_widget
            };

            f.render_widget(final_widget, key_areas[k_idx]);
        }
    }
}
