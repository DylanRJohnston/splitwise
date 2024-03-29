use async_trait::async_trait;
use color_eyre::eyre::{eyre, Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    models::{Transaction, Transactions},
    ports::budget::Budget,
};

#[derive(Debug)]
pub struct Ynab {
    budget_id: String,
    client: Client,
    bearer_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    data: Transactions,
}

impl Ynab {
    pub fn new(budget_id: String, bearer_token: String) -> Ynab {
        Ynab {
            client: Client::new(),
            budget_id,
            bearer_token,
        }
    }
}

#[async_trait]
impl Budget for Ynab {
    #[instrument]
    async fn create_transactions(&self, transactions: Vec<Transaction>) -> Result<()> {
        if transactions.is_empty() {
            return Ok(());
        }

        let response = self
            .client
            .post(format!(
                "https://api.youneedabudget.com/v1/budgets/{}/transactions",
                self.budget_id
            ))
            .bearer_auth(&self.bearer_token)
            .json(&Transactions {
                transactions: transactions.clone(),
            })
            .send()
            .await
            .wrap_err_with(|| format!("Failed to send transaction to YNAB {:?}", transactions))?;

        if response.status() != StatusCode::CREATED {
            return Err(eyre!(
                "YNAB returned {}: {}",
                response.status(),
                response.text().await?
            ));
        }

        Ok(())
    }
}
