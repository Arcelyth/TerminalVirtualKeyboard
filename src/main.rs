mod lexer;
use lexer::*;
mod parser;
use parser::*;
mod layout;
mod render;
mod error;
use error::*;
mod env;
use env::*;

use ratatui::crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*};
use rdev::{listen, EventType, Key};
use std::{
    collections::HashSet,
    io::stdout,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};


use clap::Parser as ClapParser;

#[derive(ClapParser, Debug)]
pub struct Args {
    pub path: Option<String>,
}

struct AppState {
    pressed_keys: HashSet<Key>,
    kps_events: Vec<Instant>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
    }
}

fn run() -> Result<(), AppError> {

    let mut env = Env::new();
    let args = Args::parse();
    let layout = if let Some(p) = args.path {
        let content = std::fs::read_to_string(p)?;
        let mut lexer = Lexer::new(&content);
        let tokens = lexer.tokenization();
        let mut parser = Parser::new(tokens);
        parser.parse(&mut env)?
    } else {
        return Err(AppError::WrongUsage); 
    };

    let state = Arc::new(Mutex::new(AppState {
        pressed_keys: HashSet::new(),
        kps_events: Vec::new(),
    }));


    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        listen(move |event| {
            let mut s = state_clone.lock().unwrap();
            match event.event_type {
                EventType::KeyPress(key) => {
                    if s.pressed_keys.insert(key) {
                        s.kps_events.push(Instant::now());
                    }
                }
                EventType::KeyRelease(key) => {
                    s.pressed_keys.remove(&key);
                }
                _ => {}
            }
        }).expect("The KeyEvent listening cannot be started. Please confirm if you have the necessary permissions (such as in macOS Assistive Features)");
    });

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    loop {
        terminal.draw(|f| {
            let mut s = state.lock().unwrap();
            
            // calculate KPS
            let now = Instant::now();
            s.kps_events.retain(|&t| now.duration_since(t) < Duration::from_secs(1));
            let kps = s.kps_events.len();

            render::render_ui(f, &s.pressed_keys, kps, &layout, &env);
        })?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc { break; }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}


