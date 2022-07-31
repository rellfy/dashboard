use termion::event::Key;
use crate::{State, Storage};
use crate::parse::try_parse_selected_message;

pub fn take_key(storage: &mut Storage, state: &mut State, key: Key) {
    state.should_skip_render = false;
    match key {
        Key::Ctrl('c') => state.should_exit = true,
        Key::Left => {
            if !state.should_view_message_body {
                state.set_selected_message_as_read(&storage);
            } else {
                // go back to list of messages
                state.should_view_message_body = false;
            }
        },
        Key::Right => if state.unread_messages.len() > 0 {
            if !state.should_view_message_body {
                state.cursor_height = 0;
                state.should_view_message_body = true;
                try_parse_selected_message(state);
            }
        },
        Key::Up => {
            if !state.should_view_message_body {
                state.decrease_selected_message_index()
            } else if state.cursor_height > 0 {
                state.cursor_height -= 1;
            } else {
                state.should_skip_render = true;
            }
        },
        Key::Down => {
            if !state.should_view_message_body {
                state.increase_selected_message_index()
            } else if state.cursor_height < usize::MAX {
                state.cursor_height += 1;
            } else {
                state.should_skip_render = true;
            }
        },
        _ => (),
    }
}
