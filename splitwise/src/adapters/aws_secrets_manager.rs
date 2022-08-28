use async_trait::async_trait;
use aws_config::SdkConfig;
use aws_sdk_secretsmanager::Client;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use serde_json::from_str;
use tracing::instrument;

use crate::ports::secrets::{Fetchable, Secrets};

#[derive(Debug)]
pub struct AWSSecretsManager {
    client: Client,
}

impl AWSSecretsManager {
    pub fn new(sdk_config: &SdkConfig) -> Self {
        AWSSecretsManager {
            client: Client::new(sdk_config),
        }
    }
}

#[async_trait]
impl Secrets for AWSSecretsManager {
    #[instrument]
    async fn get<F: Fetchable>(&self, key: &str) -> Result<F> {
        let output = self.client.get_secret_value().secret_id(key).send().await?;

        let data = output
            .secret_string()
            .ok_or_else(|| eyre!("Secret {} was not a string", key))?;

        from_str::<F>(data).wrap_err_with(|| eyre!("Unable to deserialize secret"))
    }
}
