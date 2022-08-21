use crate::{models::Expenses, ports::expense_tracker::ExpenseTracker};

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

struct Splitwise {
    client: Client,
    bearer_token: String,
}

pub fn new(bearer_token: String) -> impl ExpenseTracker {
    Splitwise {
        client: Client::new(),
        bearer_token,
    }
}

const EXPENSES_URL: &str = "https://secure.splitwise.com/api/v3.0/get_expenses";

#[async_trait]
impl ExpenseTracker for Splitwise {
    async fn get_all_expenses(&self) -> Result<Expenses> {
        self.client
            .get(EXPENSES_URL)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?
            .json::<Expenses>()
            .await
            .map_err(Into::into)
    }
}
