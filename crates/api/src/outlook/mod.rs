use std::error::Error;
use std::future::Future;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::StatusCode;
use serde::{Serialize, Deserialize};
use crate::mail::{Mailbox, Message};
use crate::outlook::auth::{AccessTokenRequestType, AccessTokenResponse};

pub mod auth;

const API_HOST: &'static str = "https://graph.microsoft.com";

#[derive(Serialize, Deserialize, Clone)]
pub struct OutlookMailbox {
    /// Last update timestamp.
    pub timestamp: u64,
    pub client_id: String,
    pub auth: AccessTokenResponse
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OutlookMessage {
    id: String,
    sent_date_time: String,
    has_attachments: bool,
    subject: String,
    body: OutlookMessageBody,
    body_preview: String,
    from: Recipient,
    to_recipients: Vec<Recipient>
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OutlookMessageBody {
    content_type: String,
    content: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Recipient {
    email_address: crate::mail::Recipient
}

impl OutlookMailbox {
    pub fn open(
        client_id: &str,
        auth: AccessTokenResponse
    ) -> Self {
        Self {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            client_id: client_id.to_string(),
            auth,
        }
    }

    pub async fn try_refresh_access_token(&mut self) -> bool {
        let is_expired: bool = {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let elapsed = now - self.timestamp;
            elapsed > self.auth.expires_in as u64
        };
        if !is_expired {
            return false;
        }
        let access_token = crate::outlook::auth::get_access_token(
            self.client_id.as_str(),
            AccessTokenRequestType::RefreshToken(self.auth.refresh_token.clone())
        ).await;
        self.auth = access_token;
        return true;
    }
}

#[async_trait::async_trait]
impl Mailbox for OutlookMailbox {
    fn get_id(&self) -> &str {
        self.client_id.as_str()
    }

    async fn fetch_unread(&self) -> Result<Vec<Message>, Box<dyn Error>> {
        #[derive(Deserialize)]
        struct Response {
            value: Vec<OutlookMessage>,
        }
        let api_endpoint = "/v1.0/me/mailFolders/Inbox/messages?$filter=isRead ne true&$top=1000";
        let response: Response = {
            let response = reqwest::Client::new()
                .get(format!("{}{}", API_HOST, api_endpoint))
                .header("Authorization", &self.auth.access_token)
                .send()
                .await?;
            if response.status() != StatusCode::OK {
                // todo: handle
                panic!("failed to fetch unread email");
            }
            serde_json::from_str(response.text().await?.as_str())?
        };
        let messages: Vec<Message> = response.value.iter().map(|outlook_message|
            Message {
                id: outlook_message.id.clone(),
                mailbox_id: self.get_id().to_string(),
                from: outlook_message.from.email_address.clone(),
                to: outlook_message.to_recipients.iter()
                    .map(|recipient| recipient.email_address.clone()).collect(),
                subject: outlook_message.subject.clone(),
                body: outlook_message.body.content.clone(),
            }
        ).collect();
        Ok(messages)
    }

    async fn set_as_read(self, message_id: String) {
        let api_endpoint = format!("/v1.0/me/messages/{}", message_id);
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request {
            is_read: bool,
        }
        let response = reqwest::Client::new()
            .patch(format!("{}{}", API_HOST, api_endpoint))
            .header("Authorization", &self.auth.access_token)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&Request {
                is_read: true,
            }).unwrap())
            .send()
            .await
            .unwrap();
        if response.status() != StatusCode::OK {
            panic!(format!(
                "failed to set message as read ({code}):\r\n{response}",
                code = response.status(),
                response = response.text().await.unwrap().as_str()
            ));
        }
    }
}
