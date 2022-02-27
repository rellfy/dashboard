const SCOPE_STR: &'static str = "offline_access%20user.read%20mail.readwrite%20calendars.readwrite";
const REDIRECT_URI: &'static str = "http%3A%2F%2Flocalhost%3A6767";

struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
    scope: String,
    refresh_token: String,
    id_token: String
}

/*
    Azure app client id:
    5d2e90a4-2356-4edc-ae81-80fcd4641575
*/

pub fn get_authorisation_code(client_id: &str) -> String {
    let auth_url = format!(
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?\
        client_id={}\
        &response_type=code\
        &redirect_uri={}\
        &scope={}",
        client_id,
        REDIRECT_URI,
        SCOPE_STR,
    );
    println!("{}", auth_url);
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

/*
POST /{tenant}/oauth2/v2.0/token HTTP/1.1               // Line breaks for clarity
Host: login.microsoftonline.com
Content-Type: application/x-www-form-urlencoded

client_id=6731de76-14a6-49ae-97bc-6eba6914391e
&scope=https%3A%2F%2Fgraph.microsoft.com%2Fmail.read
&code=OAAABAAAAiL9Kn2Z27UubvWFPbm0gLWQJVzCTE9UkP3pSx1aXxUjq3n8b2JRLk4OxVXr...
&redirect_uri=http%3A%2F%2Flocalhost%2Fmyapp%2F
&grant_type=authorization_code
&code_verifier=ThisIsntRandomButItNeedsToBe43CharactersLong
&client_assertion_type=urn%3Aietf%3Aparams%3Aoauth%3Aclient-assertion-type%3Ajwt-bearer
&client_assertion=eyJhbGciOiJSUzI1NiIsIng1dCI6Imd4OHRHeXN5amNScUtqRlBuZDdSRnd2d1pJMCJ9.eyJ{a lot of characters here}M8U3bSUKKJDEg
*/

pub fn get_access_token(
    client_id: &str,
    authorization_code: &str
) -> AccessTokenResponse {
    let access_url = format!(
        "https://login.microsoftonline.com/common/oauth2/v2.0/token?\
        client_id={}\
        &response_type=code\
        &redirect_uri={}\
        &scope={}\
        &code={}",
        client_id,
        REDIRECT_URI,
        SCOPE_STR,
        authorization_code
    );
}
