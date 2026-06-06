use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::config::VelipayConfig;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct VelipayClient {
    config: VelipayConfig,
    client: Client,
}

#[derive(Debug, Serialize)]
pub struct StkPushMetadata {
    pub order_id: String,
    pub customer_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStkPushRequest {
    pub amount: i64,
    pub phone_number: String,
    pub merchant_reference: String,
    pub description: String,
    pub settlement_mode: String,
    pub metadata: StkPushMetadata,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateStkPushResponse {
    #[serde(alias = "paymentID", alias = "payment_id")]
    pub payment_id: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
    pub merchant_reference: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentStatusResponse {
    #[serde(alias = "paymentID", alias = "payment_id")]
    pub payment_id: Option<String>,
    pub merchant_reference: Option<String>,
    pub status: Option<String>,
    pub amount_requested: Option<i64>,
    pub amount_received: Option<i64>,
    pub receipt_number: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessWithdrawRequest {
    pub amount: i64,
    pub source_payment_id: Option<String>,
    pub release_reference: String,
    pub destination_type: String,
    pub destination: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BusinessWithdrawResponse {
    pub payout_id: Option<i64>,
    pub release_reference: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
    pub receipt_number: Option<String>,
    pub external_request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VelipayWebhookPayload {
    pub event: String,
    pub data: VelipayWebhookData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VelipayWebhookData {
    #[serde(alias = "paymentID", alias = "payment_id")]
    pub payment_id: Option<String>,
    pub merchant_reference: Option<String>,
    pub release_reference: Option<String>,
    pub status: Option<String>,
    pub amount_requested: Option<i64>,
    pub amount_received: Option<i64>,
    pub receipt_number: Option<String>,
    pub failure_reason: Option<String>,
}

#[derive(Debug)]
pub enum VelipayError {
    Request(reqwest::Error),
    Provider(String),
}

impl std::fmt::Display for VelipayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(error) => write!(f, "{error}"),
            Self::Provider(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for VelipayError {}

impl VelipayClient {
    pub fn new(config: &VelipayConfig) -> Self {
        Self {
            config: config.clone(),
            client: Client::new(),
        }
    }

    pub async fn create_stk_push(
        &self,
        payload: &CreateStkPushRequest,
    ) -> Result<CreateStkPushResponse, VelipayError> {
        let response = self
            .client
            .post(format!("{}/api/v1/payments/stk-push", self.config.base_url.trim_end_matches('/')))
            .header("Authorization", self.authorization_header())
            .json(payload)
            .send()
            .await
            .map_err(VelipayError::Request)?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_else(|_| "unknown error".to_owned());
            return Err(VelipayError::Provider(body));
        }

        response
            .json::<CreateStkPushResponse>()
            .await
            .map_err(VelipayError::Request)
    }

    pub async fn get_payment_status(
        &self,
        payment_id: &str,
    ) -> Result<PaymentStatusResponse, VelipayError> {
        let response = self
            .client
            .get(format!(
                "{}/api/v1/payments/{}",
                self.config.base_url.trim_end_matches('/'),
                payment_id
            ))
            .header("Authorization", self.authorization_header())
            .send()
            .await
            .map_err(VelipayError::Request)?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_else(|_| "unknown error".to_owned());
            return Err(VelipayError::Provider(body));
        }

        response
            .json::<PaymentStatusResponse>()
            .await
            .map_err(VelipayError::Request)
    }

    pub async fn withdraw_business_funds(
        &self,
        payload: &BusinessWithdrawRequest,
    ) -> Result<BusinessWithdrawResponse, VelipayError> {
        let response = self
            .client
            .post(format!(
                "{}/api/v1/business/withdraw",
                self.config.base_url.trim_end_matches('/')
            ))
            .header("Authorization", self.authorization_header())
            .json(payload)
            .send()
            .await
            .map_err(VelipayError::Request)?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_else(|_| "unknown error".to_owned());
            return Err(VelipayError::Provider(body));
        }

        response
            .json::<BusinessWithdrawResponse>()
            .await
            .map_err(VelipayError::Request)
    }

    pub fn verify_signature(&self, raw_body: &[u8], header_value: &str) -> bool {
        let Some(signature) = header_value.strip_prefix("sha256=") else {
            return false;
        };

        let mut mac = match HmacSha256::new_from_slice(self.config.webhook_secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };
        mac.update(raw_body);
        let expected = hex_encode(&mac.finalize().into_bytes());
        bool::from(signature.as_bytes().ct_eq(expected.as_bytes()))
    }

    fn authorization_header(&self) -> String {
        format!(
            "Bearer {}:{}",
            self.config.api_key_id, self.config.api_key_secret
        )
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{:02x}", byte);
    }
    output
}
