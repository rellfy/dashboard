use std::error::Error;
use reqwest::StatusCode;
use serde::{Deserialize};

#[async_trait::async_trait]
pub trait Mailbox {
    fn get_id(&self) -> &str;
    async fn fetch_unread(&self) -> Result<Vec<Message>, Box<dyn Error>>;
    async fn set_as_read(self, message_id: String) -> Result<(), SetReadError>;
}

#[derive(Clone)]
pub struct Message {
    pub id: String,
    pub mailbox_id: String,
    pub subject: String,
    pub body: String,
    pub from: Recipient,
    pub to: Vec<Recipient>,
    pub date: u64,
}

#[derive(Deserialize, Clone)]
pub struct Recipient {
    pub address: String,
    pub name: String,
}

pub enum SetReadError {
    NoResponse,
    NonOkCode(StatusCode),
}