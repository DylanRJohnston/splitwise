use anyhow::Result;
use async_trait::async_trait;

use crate::models::Transaction;

#[async_trait]
pub trait Budget {
    async fn create_transactions(&self, expenses: Vec<Transaction>) -> Result<()>;
}
