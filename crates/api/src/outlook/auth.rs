use std::str::FromStr;
use serde::{Serialize, Deserialize};
use crate::outlook::auth::AccessTokenRequestType::{AuthorizationCode, RefreshToken};

const SCOPE_STR: &'static str = "\
    offline_access \
    user.read \
    mail.readwrite \
    calendars.readwrite";
const REDIRECT_URI: &'static str = "http://localhost:6767";
const API_HOST: &'static str = "https://login.microsoftonline.com";

/*
    Azure app client id:
    5d2e90a4-2356-4edc-ae81-80fcd4641575
*/

#[derive(Serialize)]
struct AuthorisationCodeRequest {
    client_id: String,
    response_type: String,
    redirect_uri: String,
    scope: String
}

#[derive(Serialize)]
struct AccessTokenRequest {
    client_id: String,
    response_type: Option<String>,
    redirect_uri: String,
    scope: String,
    code: Option<String>,
    refresh_token: Option<String>,
    grant_type: String,
}

pub enum AccessTokenRequestType {
    AuthorizationCode(String),
    RefreshToken(String)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
    pub scope: String,
    pub refresh_token: String,
}

pub fn get_authorisation_code_request_url(client_id: &str) -> String {
    let api_endpoint = "/common/oauth2/v2.0/authorize";
    let auth_url = format!(
        "{}{}?\
        client_id={}\
        &response_type=code\
        &redirect_uri={}\
        &scope={}",
        API_HOST,
        api_endpoint,
        client_id,
        REDIRECT_URI,
        SCOPE_STR,
    );
    reqwest::Url::from_str(&auth_url).unwrap().to_string()
}

pub fn get_authorisation_code() -> String {
    let mut redirect_request = crate::web::get_request();
    // Redirect request should be in the format GET /?code={} HTTP/
    let code: String = {
        let mut split = redirect_request.split("HTTP/")
            .next().unwrap().chars().as_str().split("?code=");
        if split.clone().count() != 2 {
            panic!("Invalid redirect URL. \
                It should be in the format https://localhost:port?code={}");
        }
        split.next();
        let mut chars = split.next().unwrap().chars();
        chars.next_back();
        chars.as_str().to_owned()
    };
    code
}

pub fn get_access_token(
    client_id: &str,
    request_type: AccessTokenRequestType,
) -> AccessTokenResponse {
    let api_endpoint = "/common/oauth2/v2.0/token";
    let request = AccessTokenRequest {
        client_id: client_id.to_string(),
        response_type: {
            match &request_type {
                AccessTokenRequestType::AuthorizationCode(_) => { Some("code".to_string()) }
                _ => None
            }
        },
        redirect_uri: "http://localhost:6767".to_string(),
        scope: SCOPE_STR.to_string(),
        code: {
            match &request_type {
                AccessTokenRequestType::AuthorizationCode(code) => { Some(code.clone()) }
                _ => { None }
            }
        },
        refresh_token: {
            match &request_type {
                AccessTokenRequestType::RefreshToken(refresh_token) => {
                    Some(refresh_token.clone())
                }
                _ => {
                    None
                }
            }
        },
        grant_type: {
            match &request_type {
                AccessTokenRequestType::AuthorizationCode(_) => {
                    "authorization_code"
                }
                AccessTokenRequestType::RefreshToken(_) => {
                    "refresh_token"
                }
            }
        }.to_string(),
    };
    let access_token_response: AccessTokenResponse = {
        let response = reqwest::blocking::Client::new()
            .post(format!("{}{}", API_HOST, api_endpoint))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&request)
            .send();
        let str = response.unwrap().text().unwrap();
        serde_json::from_str(str.as_str()).unwrap()
    };
    access_token_response
}
