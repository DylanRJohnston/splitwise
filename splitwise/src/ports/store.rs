use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::models::ID;

pub trait Storable = Serialize + DeserializeOwned + ID + Send + Sync + 'static;

#[async_trait]
pub trait Store {
    async fn has(&self, id: String) -> Result<bool>;
    async fn batch_has(&self, ids: &[String]) -> Result<HashSet<String>>;
    async fn add<A: Storable>(&self, item: A) -> Result<()>;
    async fn batch_add<A: Storable>(&self, items: &[A]) -> Result<()>;
}
