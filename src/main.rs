#![feature(let_chains, type_alias_impl_trait)]

mod adapters;
mod models;
mod ports;
mod usecases;

use adapters::{splitwise, sqlite::SQLite, ynab};
use anyhow::Result;

use models::{transformer::Config, YNABAccount};
use ports::secrets::Secrets;
use usecases::processor::process;

fn main() -> Result<()> {
    let secrets = adapters::env::Env;

    let expense_tracker = splitwise::new(secrets.get("SPLITWISE_API_KEY")?);
    let budget = ynab::YNAB::new(secrets.get("YNAB_BUDGET_ID")?, secrets.get("YNAB_API_KEY")?);

    let state = SQLite::new("main.db")?;

    let config = Config {
        splitwise_account: YNABAccount {
            id: secrets.get("YNAB_SPLITWISE_ACCOUNT_ID")?,
            name: secrets.get("YNAB_SPLITWISE_ACCOUNT_NAME")?,
            transfer_id: secrets.get("YNAB_SPLITWISE_TRANSFER_ID")?,
        },
        expenses_account: YNABAccount {
            id: secrets.get("YNAB_EXPENSES_ACCOUNT_ID")?,
            name: secrets.get("YNAB_EXPENSES_ACCOUNT_NAME")?,
            transfer_id: secrets.get("YNAB_EXPENSES_TRANSFER_ID")?,
        },
        splitwise_user_id: secrets.get("SPLITWISE_USER_ID")?,
    };

    process(config, expense_tracker, budget, state)
}
