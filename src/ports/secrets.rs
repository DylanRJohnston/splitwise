use std::{error::Error, str::FromStr};

use anyhow::Result;

pub trait Secrets {
    fn get<F>(&self, key: &str) -> Result<F>
    where
        F: FromStr,
        <F as FromStr>::Err: Error + Send + Sync + 'static;
}
