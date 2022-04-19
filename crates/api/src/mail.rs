use std::error::Error;
use serde::{Deserialize};

#[async_trait::async_trait]
pub trait Mailbox {
    fn get_id(&self) -> &str;
    async fn fetch_unread(&self) -> Result<Vec<Message>, Box<dyn Error>>;
    async fn set_as_read(self, message_id: String);
}

#[derive(Clone)]
pub struct Message {
    pub id: String,
    pub mailbox_id: String,
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
