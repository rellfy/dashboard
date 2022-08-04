use std::cmp::{max, min};
use std::io::Write;
use termion::terminal_size;
use api::mail::Recipient;
use crate::state::State;

pub fn print_screen(text: &str, stdout: &mut impl Write) {
    write!(
        stdout,
        "{}{}{}",
        termion::cursor::Goto(1, 1),
        termion::clear::All,
        text
    ).unwrap();
    stdout.flush().unwrap();
}

pub fn screen(state: &State, stdout: &mut impl Write) {
    print_screen("", stdout);
    if !state.is_loaded {
        print_screen("Welcome to dashboard.", stdout);
        return;
    }
    if state.should_view_message_body {
        render_message_body(state, stdout);
        return;
    }
    render_messages(&state, stdout);
}

fn render_message_body(state: &State, stdout: &mut impl Write) {
    let mut content: String = String::new();
    let selected_message_id = state.unread_messages[state.selected_message_index].id.clone();
    let body = state.parsed_message_bodies.get(&selected_message_id).unwrap().clone();
    let terminal_size = termion::terminal_size().unwrap();
    let max_rows: usize = terminal_size.1 as usize;
    let truncated = body.split("\r\n")
        .skip(state.cursor_height)
        .take(max_rows).collect::<Vec<&str>>().join("\r\n");
    content.push_str(
        &truncated
    );
    print_screen(&content, stdout);
}

fn render_messages(state: &State, stout: &mut impl Write) {
    let mut content: String = String::new();
    let terminal_size = terminal_size().unwrap();
    let terminal_width = terminal_size.0 as usize;
    let terminal_height = terminal_size.1 as usize;
    let message_height = 5;
    let messages_per_page = terminal_height / message_height;
    let to_index = max(
        min(messages_per_page, state.unread_messages.len()),
        state.selected_message_index
    );
    let from_index = if messages_per_page >= to_index {
        0
    } else {
        to_index - messages_per_page
    };
    let render_array = &state.unread_messages[from_index..to_index + 1];
    fn print_char(content: &mut String, c: char, index: usize, terminal_width: usize) {
        let mut index = index;
        while index < terminal_width {
            content.push(c);
            index += 1;
        }
    }
    for (i, message) in render_array.iter().enumerate() {
        let first_recipient = message.to.first().unwrap_or(&Recipient {
            name: "unknown".to_string(),
            address: "".to_string(),
        }).clone();
        content.push_str("\r\n\n");
        if from_index + i == state.selected_message_index {
            content.push_str(&format!("{}", termion::color::Bg(termion::color::LightBlack)));
        }
        let to_str = format!("     to: {}", &first_recipient.address);
        content.push_str(to_str.as_str());
        print_char(&mut content, ' ', to_str.len(), terminal_width);
        content.push_str("\r\n");
        let from_str = format!("   from: {} <{}>", &message.from.name, &message.from.address);
        content.push_str(from_str.as_str());
        print_char(&mut content, ' ', from_str.len(), terminal_width);
        content.push_str("\r\n");
        let subject_str = format!("subject: {} date: {}", &message.subject, message.date);
        content.push_str(subject_str.as_str());
        print_char(&mut content, ' ', subject_str.len(), terminal_width);
        content.push_str(&format!("{}", termion::color::Bg(termion::color::Reset)));
    }
    content.push_str("\r\n");
    print_screen(&content, stout);
}
