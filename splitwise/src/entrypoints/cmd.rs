#![feature(
    type_alias_impl_trait,
    generic_associated_types,
    async_closure,
    trait_alias
)]

use aws_config::meta::region::RegionProviderChain;
use color_eyre::eyre::Result;
use splitwise_ynab::{
    adapters::{dynamodb::DynamoDB, env, splitwise, ynab},
    models::{Config, YNABAccount},
    ports::secrets::Secrets,
    usecases::processor::process,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let sdk_config = aws_config::from_env()
        .region(RegionProviderChain::default_provider().or_else("ap-southeast-2"))
        .load()
        .await;

    let secrets = env::Env;

    let expense_tracker = &splitwise::new(secrets.get("SPLITWISE_API_KEY").await?);

    let budget = &ynab::Ynab::new(
        secrets.get("YNAB_BUDGET_ID").await?,
        secrets.get("YNAB_API_KEY").await?,
    );

    let records = &DynamoDB::new(&sdk_config, "splitwise");

    let config = &Config {
        splitwise: YNABAccount {
            account_id: secrets.get("YNAB_SPLITWISE_ACCOUNT_ID").await?,
            transfer_id: secrets.get("YNAB_SPLITWISE_TRANSFER_ID").await?,
        },
        expenses: YNABAccount {
            account_id: secrets.get("YNAB_EXPENSES_ACCOUNT_ID").await?,
            transfer_id: secrets.get("YNAB_EXPENSES_TRANSFER_ID").await?,
        },
        splitwise_user_id: secrets.get("SPLITWISE_USER_ID").await?,
    };

    process(config, expense_tracker, budget, records).await
}
