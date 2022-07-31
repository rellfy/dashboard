use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use crate::state::State;
use crate::storage::Storage;

pub mod storage;
mod state;
mod render;
mod parse;
mod input;
mod setup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    let mut state = State::new();
    let mut storage: Storage = storage::get();
    let mut stdout = stdout().into_raw_mode().unwrap();
    setup::setup(&mut state, &mut storage, &mut stdout).await;
    for key in stdin().keys() {
       update(&mut state, &mut storage, &mut stdout, key.unwrap());
        if state.should_exit {
            break;
        }
    }
    Ok(())
}

fn update(
    state: &mut State,
    storage: &mut Storage,
    stdout: &mut impl Write,
    key: Key
) {
    input::take_key(storage, state, key);
    render::screen(&state, stdout);
}
