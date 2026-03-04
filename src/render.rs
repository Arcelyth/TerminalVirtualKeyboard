use ratatui::layout::{Alignment, Constraint, Direction, Layout as TuiLayout};
use ratatui::{prelude::*, widgets::*};
use rdev::Key;
use std::collections::HashSet;

use crate::env::*;
use crate::layout::Layout;

pub fn render_ui(
    f: &mut Frame,
    pressed_keys: &HashSet<Key>,
    kps: usize,
    kbd_layout: &Layout,
    env: &Env,
) {
    let area = f.size();

    // Resolve Global Defaults from Env
    let default_border_color = Color::Rgb(176, 176, 176);
    let global_border_color = match env.get("border_color") {
        Some(Value::RGB(r, g, b)) => Color::Rgb(*r, *g, *b),
        _ => default_border_color,
    };

    let default_highlight = Color::Rgb(176, 176, 176);
    let global_highlight = match env.get("highlight") {
        Some(Value::RGB(r, g, b)) => Color::Rgb(*r, *g, *b),
        _ => default_highlight,
    };

    // Render Outer Container
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" Terminal Virtual Keyboard ")
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(global_border_color));

    let inner_area = outer_block.inner(area);
    f.render_widget(outer_block, area);

    // Layout: Stats Row and Keyboard Area
    let chunks = TuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner_area);

    // Render KPS
    f.render_widget(
        Paragraph::new(format!("KPS: {} ", kps))
            .alignment(Alignment::Right)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        chunks[0],
    );

    // Render Keyboard Rows
    let row_areas = TuiLayout::default()
        .direction(Direction::Vertical)
        .constraints(
            kbd_layout
                .layer
                .iter()
                .map(|_| Constraint::Length(3))
                .collect::<Vec<_>>(),
        )
        .split(chunks[1]);

    for (r_idx, row) in kbd_layout.layer.iter().enumerate() {
        let key_constraints: Vec<Constraint> = row
            .iter()
            .map(|k| Constraint::Length(k.attr.width))
            .collect();

        let key_areas = TuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints(key_constraints)
            .split(row_areas[r_idx]);

        for (k_idx, button) in row.iter().enumerate() {
            let current_border = button.attr.border_color.unwrap_or(global_border_color);
            let current_highlight = button.attr.highlight.unwrap_or(global_highlight);

            let active_bind_idx = button
                .binds
                .iter()
                .enumerate()
                .rev()
                .find(|(_, (_, key))| key.map_or(false, |k| pressed_keys.contains(&k)))
                .map(|(i, _)| i);

            let (display_name, style) = match active_bind_idx {
                Some(idx) => {
                    let name = button.binds[idx].0.as_ref();

                    let bg_color = if idx == 0 {
                        current_highlight
                    } else {
                        get_highlight(idx, env)
                    };

                    (
                        name,
                        Style::default()
                            .bg(bg_color)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
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
                        .style(if active_bind_idx.is_some() {
                            style
                        } else {
                            Style::default().fg(current_border)
                        }),
                );

            let final_render = if active_bind_idx.is_some() {
                key_widget.style(style)
            } else {
                key_widget
            };

            f.render_widget(final_render, key_areas[k_idx]);
        }
    }
}

fn get_highlight(l: usize, env: &Env) -> Color {
    let default_highlight_l2 = Color::Rgb(176, 176, 176);
    let default_highlight_l3 = Color::Rgb(176, 176, 176);
    let default_highlight_other = Color::Rgb(176, 176, 176);
    match env.get(format!("highlight_l{}", l.to_string()).as_str()) {
        Some(bc) => match bc {
            Value::RGB(r, g, b) => Color::Rgb(*r, *g, *b),
            _ => match l {
                1 => default_highlight_l2,
                2 => default_highlight_l3,
                _ => default_highlight_other,
            },
        },
        _ => match l {
            1 => default_highlight_l2,
            2 => default_highlight_l3,
            _ => default_highlight_other,
        },
    }
}
