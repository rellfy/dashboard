use std::alloc::System;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::storage::{Storage};
use api::mail::{Mailbox, Message};
use api::outlook::auth::AccessTokenRequestType;
use api::outlook::OutlookMailbox;

mod storage;

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
    println!("Welcome to dashboard.");
    let mut storage: Storage = storage::get();
    if storage.outlook.is_empty() {
        let (response, client_id) = authenticate_outlook();
        let outlook_mail = OutlookMailbox::open(
            client_id.as_str(),
            response.clone()
        );
        storage.outlook.push(outlook_mail);
        storage::set(&storage);
    }
    refresh_outlook_access_tokens(&mut storage);
    let outlook = storage.outlook.get(0).unwrap();
    let unread = outlook.fetch_unread().unwrap();
    render_messages(&unread);
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

fn render_messages(messages: &Vec<Message>) {
    for message in messages.iter() {
        let first_recipient = message.to.first().unwrap().clone();
        println!("\0");
        println!("     to: {}", &first_recipient.address);
        println!("   from: {} <{}>", message.from.name, message.from.address);
        println!("subject: {}", message.subject);
    }
    println!("\0");
}
