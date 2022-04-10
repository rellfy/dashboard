use std::str::FromStr;
use serde::{Serialize, Deserialize};

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
    response_type: String,
    redirect_uri: String,
    scope: String,
    code: String,
    grant_type: String,
}

#[derive(Serialize, Deserialize)]
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
    authorization_code: &str
) -> AccessTokenResponse {
    let api_endpoint = "/common/oauth2/v2.0/token";
    let request = AccessTokenRequest {
        client_id: client_id.to_string(),
        response_type: "code".to_string(),
        redirect_uri: "http://localhost:6767".to_string(),
        scope: SCOPE_STR.to_string(),
        code: authorization_code.to_string(),
        grant_type: "authorization_code".to_string(),
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
