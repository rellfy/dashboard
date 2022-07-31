use std::alloc::System;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::storage::{Storage};
use api::mail::{Mailbox, Message, Recipient};
use api::outlook::auth::AccessTokenRequestType;
use api::outlook::OutlookMailbox;
use std::io::{stdin, stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::terminal_size;
use tokio::task::futures;

mod storage;

struct State {
    pub is_loaded: bool,
    pub unread_messages: Vec<Message>,
    pub parsed_message_bodies: HashMap<String, String>,
    pub selected_message_index: usize,
    pub cursor_height: usize,
    pub should_view_message_body: bool,
    pub should_exit: bool,
    pub should_skip_render: bool,
}

impl State {
    pub fn set_selected_message_as_read(&mut self, storage: &Storage) {
        if self.unread_messages.len() == 0 {
            return;
        }
        let selected_message_id = self
            .unread_messages[self.selected_message_index].id.clone();
        let selected_message_mailbox_id = self
            .unread_messages[self.selected_message_index].mailbox_id.clone();
        self.unread_messages.remove(self.selected_message_index);
        self.parsed_message_bodies.remove(&selected_message_id);
        self.decrease_selected_message_index();
        let mut mailbox = storage
            .get_mailbox_by_id(selected_message_mailbox_id.as_str())
            .unwrap().clone();
        tokio::task::spawn(mailbox.set_as_read(selected_message_id));
    }

    pub fn decrease_selected_message_index(&mut self) {
        if self.selected_message_index > 0 {
            self.selected_message_index -= 1;
        }
    }

    pub fn increase_selected_message_index(&mut self) {
        let count = self.unread_messages.len();
        let min_index = if count == 0 { 0 } else { count - 1 };
        if self.selected_message_index < min_index {
            self.selected_message_index += 1;
        }
    }
}

fn print_screen(text: &str, stdout: &mut impl Write) {
    write!(
        stdout,
        "{}{}{}",
        termion::cursor::Goto(1, 1),
        termion::clear::All,
        text
    ).unwrap();
    stdout.flush().unwrap();
}

fn render_screen(state: &State, stdout: &mut impl Write) {
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

fn input_loop(storage: &mut Storage, state: &mut State, key: Key) {
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

async fn refresh_outlook_access_tokens(storage: &mut Storage) {
    let mut should_save_storage: bool = false;
    for outlook in &mut storage.outlook {
        let refreshed = outlook.try_refresh_access_token().await;
        if refreshed && !should_save_storage {
            should_save_storage = true;
        }
    }
    if should_save_storage {
        storage::set(&storage);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    let mut state: State = State {
        is_loaded: false,
        unread_messages: Vec::new(),
        parsed_message_bodies: Default::default(),
        selected_message_index: 0,
        cursor_height: 0,
        should_view_message_body: false,
        should_exit: false,
        should_skip_render: false
    };
    let mut storage: Storage = storage::get();
    let mut stdout = stdout().into_raw_mode().unwrap();
    setup(&mut state, &mut storage, &mut stdout).await;
    for key in stdin().keys() {
       update(&mut state, &mut storage, &mut stdout, key.unwrap());
        if state.should_exit {
            break;
        }
    }
    Ok(())
}

async fn setup(
    state: &mut State,
    storage: &mut Storage,
    stdout: &mut impl Write,
) {
    // add_outlook_mailbox(&mut storage).await;
    render_screen(state, stdout);
    print_screen("initialising authentication...\r\n", stdout);
    refresh_outlook_access_tokens(storage).await;
    let mut update_status_message = |i: usize, length: usize| {
        let message = format!(
            "fetching unread messages from mailboxes ({}/{})...\r\n",
            i + 1,
            length
        );
        print_screen(&message, stdout);
    };
    state.unread_messages = vec![];
    for (i, outlook_mailbox) in storage.outlook.iter().enumerate() {
        update_status_message(i, storage.outlook.len());
        state.unread_messages.append(
            &mut outlook_mailbox.fetch_unread().await.unwrap().clone(),
        );
    }
    state.is_loaded = true;
    let unread_count = state.unread_messages.len();
    state.selected_message_index = if unread_count == 0 { 0 } else { unread_count - 1 };
    render_screen(state, stdout);
}

fn update(
    state: &mut State,
    storage: &mut Storage,
    stdout: &mut impl Write,
    key: Key
) {
    input_loop(storage, state, key);
    render_screen(&state, stdout);
}

async fn add_outlook_mailbox(storage: &mut Storage) {
    let client_id: String = {
        // TODO: make client_id global per storage instead of per outlook mailbox?
        // if storage.outlook.len() > 0 {
        //     storage.outlook[0].client_id.clone()
        // } else {
            println!("Authenticating Microsoft Outlook account.");
            let register_app_txt: &str = "Register Azure app @ \
            https://docs.microsoft.com/en-us/graph/auth-register-app-v2 -- then, enter \
                    the Azure app client ID:";
            println!("{}", register_app_txt);
            let mut client_id = String::new();
            std::io::stdin().read_line(&mut client_id);
            let mut chars = client_id.chars();
            chars.next_back();
            chars.as_str().to_owned()
        // }
    };
    let response= authenticate_outlook(&client_id).await;
    let outlook_mail = OutlookMailbox::open(
        client_id.as_str(),
        response.clone()
    );
    storage.outlook.push(outlook_mail);
    storage::set(&storage);
}

async fn authenticate_outlook(client_id: &str) -> api::outlook::auth::AccessTokenResponse {
    println!("Visit the URL below to authenticate with Outlook");
    let authorisation_url = api::outlook::auth::get_authorisation_code_request_url(&client_id);
    println!("{}", authorisation_url);
    let authorisation_code = api::outlook::auth::get_authorisation_code();
    api::outlook::auth::get_access_token(
        &client_id,
        AccessTokenRequestType::AuthorizationCode(authorisation_code)
    ).await
}

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
    let terminal_size = termion::terminal_size().unwrap();
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

fn try_parse_selected_message(state: &mut State) {
    let selected_message_id = state.unread_messages[state.selected_message_index].id.clone();
    let parsed_cache = state.parsed_message_bodies.get(&selected_message_id);
    if parsed_cache.is_some() {
        return;
    }
    let parsed = parse_message_body(&state.unread_messages[state.selected_message_index].body);
    state.parsed_message_bodies
        .insert(selected_message_id, parsed.clone());
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
    let mut current_index: usize = 0;
    let mut terminal_width: usize = termion::terminal_size().unwrap().0 as usize;
    fn print_char(content: &mut String, c: char, index: usize, terminal_width: usize) {
        let mut index = index;
        while index < terminal_width {
            content.push(c);
            index += 1;
        }
    }
    for message in state.unread_messages.iter() {
        let first_recipient = message.to.first().unwrap_or(&Recipient {
            name: "unknown".to_string(),
            address: "".to_string(),
        }).clone();
        content.push_str("\r\n\n");
        if current_index == state.selected_message_index {
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
        let subject_str = format!("subject: {}", &message.subject);
        content.push_str(subject_str.as_str());
        print_char(&mut content, ' ', subject_str.len(), terminal_width);
        content.push_str(&format!("{}", termion::color::Bg(termion::color::Reset)));
        current_index += 1;
    }
    content.push_str("\r\n");
    print_screen(&content, stout);
}
