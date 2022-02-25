use api;

const SCOPE_STR: &'static str = "offline_access%20user.read%20mail.readwrite%20calendars.readwrite";

/*
    Azure app client id:
    5d2e90a4-2356-4edc-ae81-80fcd4641575
*/

fn main() {
    let authorisation_code = get_authorisation_code();
    println!("code: {}", authorisation_code);
}

fn get_authorisation_code() -> String {
    let register_app_txt: &str = "Register Azure app @ \
        https://docs.microsoft.com/en-us/graph/auth-register-app-v2";
    println!("Register app at {}", register_app_txt);
    println!("Enter Azure app client ID:");
    let mut client_id = String::new();
    std::io::stdin().read_line(&mut client_id);
    client_id = {
        let mut chars = client_id.chars();
        chars.next_back();
        chars.as_str().to_owned()
    };
    let auth_url = format!(
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?\
        client_id={}\
        &response_type=code\
        &redirect_uri=http%3A%2F%2Flocalhost%3A6767\
        &scope={}",
        &client_id,
        SCOPE_STR
    );
    println!("Visit the following URL to authenticate: {}", auth_url);
    let mut redirect_request = api::web::get_request();
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
