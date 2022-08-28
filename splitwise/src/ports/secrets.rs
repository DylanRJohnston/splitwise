use std::{error::Error, str::FromStr};

use async_trait::async_trait;
use color_eyre::eyre::Result;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use tracing::instrument;

pub trait Fetchable = FromStr + Send + Sync + DeserializeOwned
where <Self as FromStr>::Err: Error + Send + Sync + 'static;

#[async_trait]
pub trait Secrets: Sync + Send + Debug {
    async fn get<F: Fetchable>(&self, key: &str) -> Result<F>;
}

#[derive(Debug)]
struct Combined<First: Secrets, Second: Secrets> {
    first: First,
    second: Second,
}

#[async_trait]
impl<First: Secrets, Second: Secrets> Secrets for Combined<First, Second> {
    #[instrument]
    async fn get<F: Fetchable>(&self, key: &str) -> Result<F> {
        if let Ok(value) = self.first.get::<F>(key).await {
            return Ok(value);
        }

        self.second.get::<F>(key).await
    }
}

pub fn combine(first: impl Secrets, second: impl Secrets) -> impl Secrets {
    Combined { first, second }
}
