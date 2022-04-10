use std::alloc::System;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::storage::{OutlookStorage, Storage};
use api::mail::{Mailbox, MailboxAuthenticator, Message};
use api::outlook::auth::AccessTokenRequestType;
use api::outlook::OutlookMailbox;

mod storage;

fn main() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("Welcome to dashboard.");
    let mut storage: Storage = storage::get();
    if storage.outlook.is_none() {
        let (response, client_id) = authenticate_outlook();
        storage.outlook = Some(OutlookStorage {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            client_id,
            authentication_response: response
        });
        storage::set(&storage);
    }
    let outlook: Box<dyn Mailbox> = OutlookMailbox
        ::open(storage.outlook.unwrap().authentication_response.clone());
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
