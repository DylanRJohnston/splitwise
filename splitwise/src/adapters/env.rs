use crate::ports::secrets::Secrets;
use anyhow::{Context, Result};
use std::{env, error::Error, str::FromStr};

pub struct Env;

impl Secrets for Env {
    fn get<F>(&self, key: &str) -> Result<F>
    where
        F: FromStr,
        <F as FromStr>::Err: Error + Send + Sync + 'static,
    {
        let raw =
            env::var(key).with_context(|| format!("Failed to fetch {} from environment", key))?;

        let parsed = raw.parse()?;

        Ok(parsed)
    }
}
