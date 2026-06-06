use sea_orm::DatabaseConnection;

use crate::{config::Config, mailer::ResendMailer, storage::SupabaseStorage, velipay::VelipayClient};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: DatabaseConnection,
    pub storage: Option<SupabaseStorage>,
    pub mailer: Option<ResendMailer>,
    pub velipay: Option<VelipayClient>,
}

impl AppState {
    pub fn new(config: Config, db: DatabaseConnection) -> Self {
        let storage = config
            .supabase_storage
            .as_ref()
            .map(SupabaseStorage::new);
        let mailer = config.resend.as_ref().map(ResendMailer::new);
        let velipay = config.velipay.as_ref().map(VelipayClient::new);

        Self {
            config,
            db,
            storage,
            mailer,
            velipay,
        }
    }
}
