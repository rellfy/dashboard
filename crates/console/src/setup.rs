use std::io::Write;
use api::mail::Mailbox;
use api::outlook::auth::AccessTokenRequestType;
use api::outlook::OutlookMailbox;
use crate::{render, State, Storage};
use crate::render::print_screen;
use crate::storage;

pub async fn setup(
    state: &mut State,
    storage: &mut Storage,
    stdout: &mut impl Write,
) {
    // add_outlook_mailbox(&mut storage).await;
    render::screen(state, stdout);
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
    render::screen(state, stdout);
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
        std::io::stdin().read_line(&mut client_id).expect("Failed to read client_id");
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
