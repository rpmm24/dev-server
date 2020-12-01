use super::*;

#[derive(Clone)]
pub struct StatusWriter {
    webhook: Option<webhook::Client>,
}

impl StatusWriter {
    pub fn none() -> StatusWriter {
        StatusWriter { webhook: None }
    }

    pub fn write(&self, message: impl Into<String>) {
        if let Some(webhook) = &self.webhook {
            let webhook = webhook.clone();
            let message = message.into();

            tokio::spawn(async move {
                let result = webhook.post(&webhook::Payload {
                    content: message,
                    username: None,
                    avatar_url: None,
                }).await;

                if let Err(err) = result {
                    eprintln!("failed to post to webhook: {:?}", err);
                }
            });
        }
    }
}

impl From<webhook::Client> for StatusWriter {
    fn from(webhook: webhook::Client) -> Self {
        StatusWriter {
            webhook: Some(webhook)
        }
    }
}
