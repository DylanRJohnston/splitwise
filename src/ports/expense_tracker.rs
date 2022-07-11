use crate::models::Expenses;
use anyhow::Result;

pub trait ExpenseTracker {
    fn get_all_expenses(&self) -> Result<Expenses>;
}
