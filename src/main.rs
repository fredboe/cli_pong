use crate::state::GameState;
use crate::utils::GameLoop;
use clap::Parser;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::collections::HashMap;
use std::io;
use std::time::Duration;

mod state;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Width of the game window
    #[arg(short, long, default_value_t = 60)]
    width: usize,

    /// Height of the game window
    #[arg(short, long, default_value_t = 18)]
    height: usize,

    /// Defines how much longer the player should be in the top direction.
    #[arg(short, long, default_value_t = 1)]
    up_extend_player_height: usize,

    /// Defines how much longer the player should be in the bottom direction.
    #[arg(short, long, default_value_t = 1)]
    down_extend_player_height: usize,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    enable_raw_mode()?;

    let mut game_state = GameState::new(
        args.width,
        args.height,
        args.up_extend_player_height,
        args.down_extend_player_height,
    );
    for _ in GameLoop::from_fps(10) {
        let key_events = get_pressed_keys().unwrap_or(HashMap::new());

        if let Some(key_event) = key_events.get(&KeyCode::Char('c')) {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                break;
            }
        }

        game_state.update(key_events, Duration::from_millis(100));
        game_state
            .display()
            .unwrap_or_else(|_| println!("Failed to display!"));
    }

    disable_raw_mode()?;
    Ok(())
}

fn get_pressed_keys() -> io::Result<HashMap<KeyCode, KeyEvent>> {
    let mut pressed_keys = HashMap::new();

    while poll(Duration::from_millis(20))? {
        if let Event::Key(key_event) = read()? {
            pressed_keys.insert(key_event.code, key_event);
        }
    }

    Ok(pressed_keys)
}
