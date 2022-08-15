use serde::{Deserialize, Serialize};

use super::{Expense, ID};

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
}

impl ID for Record {
    type ID = i64;

    fn id(&self) -> Self::ID {
        self.id
    }
}

impl From<&Expense> for Record {
    fn from(expense: &Expense) -> Self {
        Self { id: expense.id }
    }
}
