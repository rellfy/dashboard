use termion::terminal_size;
use crate::State;

fn remove_empty_lines(string: &mut String) {
    let mut index_option = string.find(" \r\n");
    while index_option != None {
        let index = index_option.unwrap();
        string.replace_range(index..index + 1, "");
        index_option = string.find(" \r\n");
    }
    index_option = string.find("\r\n\r\n");
    while index_option != None {
        let index = index_option.unwrap();
        string.replace_range(index..index + 2, "");
        index_option = string.find("\r\n\r\n");
    }
}

fn parse_message_body(body: &str) -> String {
    let terminal_size = terminal_size().unwrap();
    let bytes = body.as_bytes();
    let mut text = html2text
    ::from_read(bytes, terminal_size.0 as usize)
        .replace("\n", "\r\n")
        .replace("─", "")
        .replace("┴", "")
        .replace("┬", "");
    // .replace("│", ""); // TODO: use this char to parse columns into rows.
    remove_empty_lines(&mut text);
    text
}

pub fn try_parse_selected_message(state: &mut State) {
    let selected_message_id = state.unread_messages[state.selected_message_index].id.clone();
    let parsed_cache = state.parsed_message_bodies.get(&selected_message_id);
    if parsed_cache.is_some() {
        return;
    }
    let parsed = parse_message_body(&state.unread_messages[state.selected_message_index].body);
    state.parsed_message_bodies
        .insert(selected_message_id, parsed.clone());
}