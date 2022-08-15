use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use crate::models::ID;

#[async_trait]
pub trait Set<A: ID> {
    async fn has(&self, key: A) -> Result<bool>;
    async fn batch_has(&self, key: &[A]) -> Result<HashMap<A::ID, A>>;
    async fn add(&self, key: A) -> Result<()>;
    async fn batch_add(&self, key: &[A]) -> Result<()>;
}
