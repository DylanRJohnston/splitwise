use super::{Cents, Expense, Repayment, Share, User, ID};
use own::own;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
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

impl ID for Record {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl From<&Expense> for Record {
    #[allow(clippy::redundant_clone)]
    fn from(expense: &Expense) -> Self {
        own! {Self {
            id: expense.id.to_string(),
            group_id: expense.group_id,
            description: expense.description,
            payment: expense.payment,
            cost: expense.cost,
            repayments: expense.repayments,
            date: expense.date,
            created_at: expense.created_at,
            created_by: expense.created_by,
            updated_at: expense.updated_at,
            users: expense.users,
        }}
    }
}
