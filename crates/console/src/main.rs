use std::alloc::System;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::storage::{OutlookStorage, Storage};
use api;

mod storage;

fn main() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("Welcome to dashboard.");
    let mut storage: Storage = storage::get();
    if storage.outlook.is_none() {
        let response = authenticate_outlook();
        storage.outlook = Some(OutlookStorage {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            authentication_response: response
        });
        storage::set(&storage);
    }
}

fn authenticate_outlook() -> api::outlook::auth::AccessTokenResponse {
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
    api::outlook::auth::get_access_token(&client_id, &authorisation_code)
}
