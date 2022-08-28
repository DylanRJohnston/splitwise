use crate::ports::secrets::{Fetchable, Secrets};
use async_trait::async_trait;
use color_eyre::{eyre::Context, Result};
use std::env;
use tracing::instrument;

#[derive(Debug)]
pub struct Env;

#[async_trait]
impl Secrets for Env {
    #[instrument]
    async fn get<F: Fetchable>(&self, key: &str) -> Result<F> {
        let raw =
            env::var(key).with_context(|| format!("Failed to fetch {} from environment", key))?;

        let parsed = raw.parse()?;

        Ok(parsed)
    }
}
