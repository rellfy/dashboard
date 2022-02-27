use api;

fn main() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("Welcome to dashboard.");
    authenticate_outlook();
}

fn authenticate_outlook() {
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
    let authorisation_code = api::outlook::get_authorisation_code(client_id.as_str());
    println!("code: {}", authorisation_code);
    let
}
