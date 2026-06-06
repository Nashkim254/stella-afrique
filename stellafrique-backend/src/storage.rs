use reqwest::Client;

use crate::config::SupabaseStorageConfig;

#[derive(Clone)]
pub struct SupabaseStorage {
    client: Client,
    upload_base_url: String,
    public_base_url: String,
    service_role_key: String,
}

impl SupabaseStorage {
    pub fn new(config: &SupabaseStorageConfig) -> Self {
        let base_project_url = config.project_url.trim_end_matches('/');
        let public_base_url = format!(
            "{}/storage/v1/object/public/{}",
            base_project_url,
            config.storage_bucket,
        );
        let upload_base_url = format!(
            "{}/storage/v1/object/{}",
            base_project_url,
            config.storage_bucket,
        );

        Self {
            client: Client::new(),
            upload_base_url,
            public_base_url,
            service_role_key: config.service_role_key.clone(),
        }
    }

    pub async fn upload_public_image(
        &self,
        object_path: &str,
        bytes: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<String, reqwest::Error> {
        let request = self
            .client
            .post(format!("{}/{}", self.upload_base_url, object_path))
            .header("authorization", format!("Bearer {}", self.service_role_key))
            .header("apikey", &self.service_role_key)
            .header("x-upsert", "false")
            .header(
                reqwest::header::CONTENT_TYPE,
                content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            )
            .body(bytes);

        request.send().await?.error_for_status()?;

        Ok(format!("{}/{}", self.public_base_url, object_path))
    }
}
