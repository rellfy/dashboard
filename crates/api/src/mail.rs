pub trait Mailbox {
    fn fetch_unread() -> Vec<Mail>;
}

pub struct Mail {
    subject: String,
    body: String
}
