#![feature(
    let_chains,
    type_alias_impl_trait,
    generic_associated_types,
    async_closure,
    trait_alias
)]

mod adapters;
mod models;
mod ports;
mod usecases;

use adapters::{dynamodb::DynamoDB, splitwise, ynab};
use anyhow::Result;

use models::{transformer::Config, YNABAccount};
use ports::secrets::Secrets;
use usecases::processor::process;

#[tokio::main]
async fn main() -> Result<()> {
    let secrets = adapters::env::Env;

    let expense_tracker = splitwise::new(secrets.get("SPLITWISE_API_KEY")?);
    let budget = ynab::Ynab::new(secrets.get("YNAB_BUDGET_ID")?, secrets.get("YNAB_API_KEY")?);

    let state = DynamoDB::new("splitwise".to_owned()).await;

    let config = Config {
        splitwise: YNABAccount {
            account_id: secrets.get("YNAB_SPLITWISE_ACCOUNT_ID")?,
            transfer_id: secrets.get("YNAB_SPLITWISE_TRANSFER_ID")?,
        },
        expenses: YNABAccount {
            account_id: secrets.get("YNAB_EXPENSES_ACCOUNT_ID")?,
            transfer_id: secrets.get("YNAB_EXPENSES_TRANSFER_ID")?,
        },
        splitwise_user_id: secrets.get("SPLITWISE_USER_ID")?,
    };

    process(config, expense_tracker, budget, state).await
}
