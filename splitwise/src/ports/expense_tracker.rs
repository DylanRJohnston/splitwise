use crate::models::Expenses;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ExpenseTracker {
    async fn get_all_expenses(&self) -> Result<Expenses>;
}
