use std::alloc::System;
use std::cmp::{max, min};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::storage::{Storage};
use api::mail::{Mailbox, Message};
use api::outlook::auth::AccessTokenRequestType;
use api::outlook::OutlookMailbox;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tokio::task::futures;

mod storage;

struct State {
    pub is_loaded: bool,
    pub unread_messages: Vec<Message>,
    pub selected_message_index: usize,
    pub should_view_message_body: bool,
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
        print_screen("view msg body", stdout);
        return;
    }
    render_messages(&state, stdout);
}

fn input_loop(mut storage: Storage, mut state: State, mut stdout: impl Write) {
    let stdin = stdin();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Ctrl('c') => break,
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
                    state.should_view_message_body = true;
                }
            },
            Key::Up => state.decrease_selected_message_index(),
            Key::Down => state.increase_selected_message_index(),
            _ => (),
        }
        render_screen(&state, &mut stdout);
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
        selected_message_index: 0,
        should_view_message_body: false,
    };
    let mut stdout = stdout().into_raw_mode().unwrap();
    render_screen(&state, &mut stdout);
    let mut storage: Storage = storage::get();
    handle_authentication(&mut storage).await;
    let outlook = storage.outlook.get(0).unwrap();
    state.unread_messages = outlook.fetch_unread().await.unwrap();
    state.is_loaded = true;
    let unread_count = state.unread_messages.len();
    state.selected_message_index = if unread_count == 0 { 0 } else { unread_count - 1 };
    render_screen(&state, &mut stdout);
    input_loop(storage, state, stdout);
    Ok(())
}

async fn handle_authentication(storage: &mut Storage) {
    if storage.outlook.is_empty() {
        let (response, client_id) = authenticate_outlook().await;
        let outlook_mail = OutlookMailbox::open(
            client_id.as_str(),
            response.clone()
        );
        storage.outlook.push(outlook_mail);
        storage::set(&storage);
    }
    refresh_outlook_access_tokens(storage).await;
}

async fn authenticate_outlook() -> (api::outlook::auth::AccessTokenResponse, String) {
    println!("Authenticating Microsoft Outlook account.");
    let register_app_txt: &str = "Register Azure app @ \
        https://docs.microsoft.com/en-us/graph/auth-register-app-v2 -- then, enter \
                the Azure app client ID:";
    println!("{}", register_app_txt);
    let mut client_id = String::new();
    std::io::stdin().read_line(&mut client_id);
    client_id = {
        let mut chars = client_id.chars();
        chars.next_back();
        chars.as_str().to_owned()
    };
    println!("Visit the URL below to authenticate with Outlook");
    let authorisation_url = api::outlook::auth::get_authorisation_code_request_url(&client_id);
    println!("{}", authorisation_url);
    let authorisation_code = api::outlook::auth::get_authorisation_code();
    let access_token = api::outlook::auth::get_access_token(
        &client_id,
        AccessTokenRequestType::AuthorizationCode(authorisation_code)
    ).await;
    (access_token, client_id)
}

fn render_messages(state: &State, stout: &mut impl Write) {
    let mut content: String = "".to_string();
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
        let first_recipient = message.to.first().unwrap().clone();
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
    print_screen(content.as_str(), stout);
}
