use anyhow::{bail, Result};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    models::{Transaction, Transactions},
    ports::budget::Budget,
};

pub struct YNAB {
    budget_id: String,
    client: Client,
    bearer_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    data: Transactions,
}

impl YNAB {
    pub fn new(budget_id: String, bearer_token: String) -> YNAB {
        YNAB {
            client: Client::new(),
            budget_id,
            bearer_token,
        }
    }
}

#[async_trait]
impl Budget for YNAB {
    async fn create_transactions(&self, transactions: Vec<Transaction>) -> Result<()> {
        if transactions.len() == 0 {
            return Ok(());
        }

        let response = self
            .client
            .post(format!(
                "https://api.youneedabudget.com/v1/budgets/{}/transactions",
                self.budget_id
            ))
            .bearer_auth(&self.bearer_token)
            .json(&Transactions { transactions })
            .send()
            .await?;

        if response.status() != StatusCode::CREATED {
            bail!(
                "YNAB returned {}: {}",
                response.status(),
                response.text().await?
            );
        }

        println!("{:?}", response.text().await?);
        Ok(())
    }
}
