use anyhow::{bail, Result};
use reqwest::{blocking::Client, StatusCode};
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

impl Budget for YNAB {
    fn create_transactions(&self, transactions: Vec<Transaction>) -> Result<()> {
        let response = self
            .client
            .post(format!(
                "https://api.youneedabudget.com/v1/budgets/{}/transactions",
                self.budget_id
            ))
            .bearer_auth(&self.bearer_token)
            .json(&Transactions { transactions })
            .send()?;

        if response.status() != StatusCode::CREATED {
            bail!("YNAB returned {}: {}", response.status(), response.text()?);
        }

        println!("{:?}", response.text());
        Ok(())
    }
}
