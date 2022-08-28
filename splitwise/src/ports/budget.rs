use async_trait::async_trait;
use color_eyre::eyre::Result;
use std::fmt::Debug;

use crate::models::Transaction;

#[async_trait]
pub trait Budget: Debug {
    async fn create_transactions(&self, expenses: Vec<Transaction>) -> Result<()>;
}
