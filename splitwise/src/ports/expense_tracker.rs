use crate::models::Expenses;
use async_trait::async_trait;
use color_eyre::eyre::Result;
use std::fmt::Debug;

#[async_trait]
pub trait ExpenseTracker: Debug {
    async fn get_all_expenses(&self) -> Result<Expenses>;
}
