use serde::{Deserialize, Serialize};

use super::{Expense, ID};

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
}

impl ID for Record {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl From<&Expense> for Record {
    fn from(expense: &Expense) -> Self {
        Self { id: expense.id }
    }
}
