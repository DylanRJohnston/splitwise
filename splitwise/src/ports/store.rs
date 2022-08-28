use std::collections::HashSet;

use async_trait::async_trait;
use color_eyre::eyre::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::models::ID;

pub trait Storable = Debug + Serialize + DeserializeOwned + ID + Send + Sync + 'static;

#[async_trait]
pub trait Store: Debug {
    async fn has(&self, id: String) -> Result<bool>;
    async fn batch_has(&self, ids: &[String]) -> Result<HashSet<String>>;
    async fn add<A: Storable>(&self, item: A) -> Result<()>;
    async fn batch_add<A: Storable>(&self, items: &[A]) -> Result<()>;
}
