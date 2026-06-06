use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub app_env: String,
    pub host: String,
    pub port: u16,
    pub frontend_origin: String,
    pub database_url: String,
    pub payment_test_amount_override: Option<i64>,
    pub supabase_storage: Option<SupabaseStorageConfig>,
    pub resend: Option<ResendConfig>,
    pub velipay: Option<VelipayConfig>,
    pub admin_auth: Option<AdminAuthConfig>,
    pub customer_auth: Option<CustomerAuthConfig>,
}

#[derive(Clone, Debug)]
pub struct AdminAuthConfig {
    pub session_secret: String,
    pub bootstrap_email: Option<String>,
    pub bootstrap_password: Option<String>,
    pub bootstrap_name: String,
}

#[derive(Clone, Debug)]
pub struct CustomerAuthConfig {
    pub session_secret: String,
}

#[derive(Clone, Debug)]
pub struct SupabaseStorageConfig {
    pub project_url: String,
    pub storage_bucket: String,
    pub service_role_key: String,
}

#[derive(Clone, Debug)]
pub struct ResendConfig {
    pub api_key: String,
    pub from_email: String,
    pub from_name: String,
    pub reply_to: Option<String>,
    pub notification_emails: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct VelipayConfig {
    pub base_url: String,
    pub api_key_id: String,
    pub api_key_secret: String,
    pub webhook_secret: String,
    pub webhook_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let project_url = env::var("SUPABASE_PROJECT_URL").ok();
        let storage_bucket = env::var("SUPABASE_STORAGE_BUCKET").ok();
        let service_role_key = env::var("SUPABASE_SERVICE_ROLE_KEY").ok();
        let resend_api_key = env::var("RESEND_API_KEY").ok();
        let resend_from_email = env::var("RESEND_FROM_EMAIL").ok();
        let resend_from_name = env::var("RESEND_FROM_NAME").ok();
        let resend_reply_to = env::var("RESEND_REPLY_TO").ok();
        let resend_notification_emails = env::var("RESEND_NOTIFICATION_EMAILS").ok();
        let velipay_base_url = env::var("VELIPAY_BASE_URL").ok();
        let velipay_api_key_id = env::var("VELIPAY_API_KEY_ID").ok();
        let velipay_api_key_secret = env::var("VELIPAY_API_KEY_SECRET").ok();
        let velipay_webhook_secret = env::var("VELIPAY_WEBHOOK_SECRET").ok();
        let velipay_webhook_url = env::var("VELIPAY_WEBHOOK_URL").ok();
        let admin_email = env::var("ADMIN_EMAIL").ok();
        let admin_password = env::var("ADMIN_PASSWORD").ok();
        let admin_session_secret = env::var("ADMIN_SESSION_SECRET").ok();
        let customer_session_secret = env::var("CUSTOMER_SESSION_SECRET").ok();

        let supabase_storage = match (project_url, storage_bucket, service_role_key) {
            (
                Some(project_url),
                Some(storage_bucket),
                Some(service_role_key),
            ) => Some(SupabaseStorageConfig {
                project_url,
                storage_bucket,
                service_role_key,
            }),
            _ => None,
        };

        let resend = match (resend_api_key, resend_from_email, resend_from_name) {
            (Some(api_key), Some(from_email), Some(from_name)) => Some(ResendConfig {
                api_key,
                from_email,
                from_name,
                reply_to: resend_reply_to,
                notification_emails: resend_notification_emails
                    .unwrap_or_default()
                    .split(',')
                    .map(str::trim)
                    .filter(|email| !email.is_empty())
                    .map(ToString::to_string)
                    .collect(),
            }),
            _ => None,
        };

        let velipay = match (
            velipay_base_url,
            velipay_api_key_id,
            velipay_api_key_secret,
            velipay_webhook_secret,
        ) {
            (
                Some(base_url),
                Some(api_key_id),
                Some(api_key_secret),
                Some(webhook_secret),
            ) => Some(VelipayConfig {
                base_url,
                api_key_id,
                api_key_secret,
                webhook_secret,
                webhook_url: velipay_webhook_url,
            }),
            _ => None,
        };

        let admin_auth = admin_session_secret.map(|session_secret| AdminAuthConfig {
            session_secret,
            bootstrap_email: admin_email,
            bootstrap_password: admin_password,
            bootstrap_name: env::var("ADMIN_FULL_NAME")
                .unwrap_or_else(|_| "Stellafrique Owner".to_owned()),
        });

        let customer_auth = customer_session_secret.map(|session_secret| CustomerAuthConfig {
            session_secret,
        });

        Ok(Self {
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("APP_PORT must be a valid port number")?,
            frontend_origin: env::var("FRONTEND_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL is required to start the backend")?,
            payment_test_amount_override: env::var("PAYMENT_TEST_AMOUNT_OVERRIDE")
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .filter(|value| *value > 0),
            supabase_storage,
            resend,
            velipay,
            admin_auth,
            customer_auth,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
