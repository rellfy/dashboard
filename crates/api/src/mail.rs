use std::error::Error;
use serde::{Deserialize};

pub trait Mailbox {
    fn fetch_unread(&self) -> Result<Vec<Message>, Box<dyn Error>>;
    fn set_as_read(&self, message: &Message) -> Result<bool, Box<dyn Error>>;
}

pub struct Message {
    pub id: String,
    pub subject: String,
    pub body: String,
    pub from: Recipient,
    pub to: Vec<Recipient>
}

#[derive(Deserialize, Clone)]
pub struct Recipient {
    pub address: String,
    pub name: String,
}
