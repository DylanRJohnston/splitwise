use crate::{models::Expenses, ports::expense_tracker::ExpenseTracker};

use async_trait::async_trait;
use color_eyre::eyre::{Context, Result};
use reqwest::Client;
use tracing::instrument;

#[derive(Debug)]
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
    #[instrument]
    async fn get_all_expenses(&self) -> Result<Expenses> {
        self.client
            .get(EXPENSES_URL)
            .bearer_auth(&self.bearer_token)
            .send()
            .await
            .wrap_err("Failed to retrieve expenses from splitwise")?
            .json::<Expenses>()
            .await
            .wrap_err("Failed to deserialize response from splitwise")
    }
}
