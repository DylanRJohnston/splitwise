use super::{Cents, ID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Repayment {
    pub from: i64,
    pub to: i64,
    pub amount: Cents,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
}

impl User {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    pub user: User,
    pub user_id: i64,
    pub paid_share: Cents,
    pub owed_share: Cents,
    pub net_balance: Cents,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Expense {
    pub id: i64,
    pub group_id: i64,
    pub description: String,
    pub payment: bool,
    pub cost: Cents,
    pub repayments: Vec<Repayment>,
    pub date: String,
    pub created_at: String,
    pub created_by: User,
    pub updated_at: String,
    pub users: Vec<Share>,
}

impl ID for Expense {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Expenses {
    pub expenses: Vec<Expense>,
}
