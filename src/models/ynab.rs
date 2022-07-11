use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Transaction {
    pub account_id: String,
    pub date: String,
    pub amount: i64,
    pub payee_id: Option<String>,
    pub payee_name: Option<String>,
    pub memo: String,
    pub cleared: String,
    pub approved: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}
