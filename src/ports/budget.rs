use anyhow::Result;

use crate::models::Transaction;

pub trait Budget {
    fn create_transactions(&self, expenses: Vec<Transaction>) -> Result<()>;
}
