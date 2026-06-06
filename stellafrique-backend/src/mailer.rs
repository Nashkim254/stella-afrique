use reqwest::Client;
use serde::Serialize;

use crate::config::ResendConfig;

const RESEND_API_URL: &str = "https://api.resend.com/emails";

#[derive(Clone)]
pub struct ResendMailer {
    client: Client,
    api_key: String,
    from: String,
    reply_to: Option<String>,
    notification_emails: Vec<String>,
}

#[derive(Serialize)]
struct ResendEmailPayload<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    html: &'a str,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<&'a str>,
}

impl ResendMailer {
    pub fn new(config: &ResendConfig) -> Self {
        Self {
            client: Client::new(),
            api_key: config.api_key.clone(),
            from: format!("{} <{}>", config.from_name, config.from_email),
            reply_to: config.reply_to.clone(),
            notification_emails: config.notification_emails.clone(),
        }
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        text: &str,
        idempotency_key: &str,
    ) -> Result<(), reqwest::Error> {
        self.send_email_to_many(&[to], subject, html, text, idempotency_key)
            .await
    }

    pub async fn send_notification_email(
        &self,
        subject: &str,
        html: &str,
        text: &str,
        idempotency_key: &str,
    ) -> Result<(), reqwest::Error> {
        if self.notification_emails.is_empty() {
            return Ok(());
        }

        let recipients = self
            .notification_emails
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();

        self.send_email_to_many(&recipients, subject, html, text, idempotency_key)
            .await
    }

    async fn send_email_to_many(
        &self,
        to: &[&str],
        subject: &str,
        html: &str,
        text: &str,
        idempotency_key: &str,
    ) -> Result<(), reqwest::Error> {
        let payload = ResendEmailPayload {
            from: &self.from,
            to: to.to_vec(),
            subject,
            html,
            text,
            reply_to: self.reply_to.as_deref(),
        };

        self.client
            .post(RESEND_API_URL)
            .bearer_auth(&self.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("Idempotency-Key", idempotency_key)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
