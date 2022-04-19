use std::alloc::System;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::storage::{Storage};
use api::mail::{Mailbox, Message};
use api::outlook::auth::AccessTokenRequestType;
use api::outlook::OutlookMailbox;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod storage;

struct State<'a> {
    pub is_loaded: bool,
    pub unread_messages: Vec<&'a Message>,
    pub should_mark_message_as_read: bool,
    pub should_view_message_body: bool,
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
    if state.should_mark_message_as_read {
        print_screen("marked as read", stdout);
        return;
    }
    if state.should_view_message_body {
        print_screen("view msg body", stdout);
        return;
    }
    render_messages(&state.unread_messages, stdout);
}

fn input_loop(mut state: State, mut stdout: impl Write) {
    let stdin = stdin();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Ctrl('c') => break,
            Key::Left => state.should_mark_message_as_read = !state.should_mark_message_as_read,
            Key::Right => state.should_view_message_body = !state.should_view_message_body,
            _ => (),
        }
        render_screen(&state, &mut stdout);
    }
}

fn refresh_outlook_access_tokens(storage: &mut Storage) {
    let mut should_save_storage: bool = false;
    for outlook in &mut storage.outlook {
        let refreshed = outlook.try_refresh_access_token();
        if refreshed && !should_save_storage {
            should_save_storage = true;
        }
    }
    if should_save_storage {
        storage::set(&storage);
    }
}

fn main() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    let mut state: State = State {
        is_loaded: false,
        unread_messages: Vec::new(),
        should_mark_message_as_read: false,
        should_view_message_body: false,
    };
    let mut stdout = stdout().into_raw_mode().unwrap();
    render_screen(&state, &mut stdout);
    let mut storage: Storage = storage::get();
    handle_authentication(&mut storage);
    let outlook = storage.outlook.get(0).unwrap();
    let unread = outlook.fetch_unread().unwrap();
    state.unread_messages = unread.iter().map(|m| m).collect();
    state.is_loaded = true;
    render_screen(&state, &mut stdout);
    input_loop(state, stdout);
}

fn handle_authentication(storage: &mut Storage) {
    if storage.outlook.is_empty() {
        let (response, client_id) = authenticate_outlook();
        let outlook_mail = OutlookMailbox::open(
            client_id.as_str(),
            response.clone()
        );
        storage.outlook.push(outlook_mail);
        storage::set(&storage);
    }
    refresh_outlook_access_tokens(storage);
}

fn authenticate_outlook() -> (api::outlook::auth::AccessTokenResponse, String) {
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
    );
    (access_token, client_id)
}

fn render_messages(messages: &Vec<&Message>, stout: &mut impl Write) {
    let mut content: String = "".to_string();
    for message in messages.iter() {
        let first_recipient = message.to.first().unwrap().clone();
        content.push_str("\r\n\r\n");
        content.push_str("     to: ");
        content.push_str(&first_recipient.address);
        content.push_str("\r\n");
        content.push_str("   from: ");
        content.push_str(&message.from.name);
        content.push_str(" <");
        content.push_str(&message.from.address);
        content.push_str("\r\n");
        content.push_str("subject: ");
        content.push_str(&message.subject);
    }
    content.push_str("\r\n");
    print_screen(content.as_str(), stout);
}
