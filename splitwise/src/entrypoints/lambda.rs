#![feature(
    type_alias_impl_trait,
    generic_associated_types,
    async_closure,
    trait_alias
)]

use std::str::FromStr;

use aws_config::meta::region::RegionProviderChain;
use color_eyre::{
    config::{HookBuilder, Theme},
    eyre::{eyre, WrapErr},
    Result,
};
use lambda_runtime::{run, service_fn, LambdaEvent};
use serde::Deserialize;
use serde_json::{from_str, Error};
use splitwise_ynab::{
    adapters::{
        aws_secrets_manager::AWSSecretsManager, dynamodb::DynamoDB, env, splitwise, ynab::Ynab,
    },
    models::{Config, YNABAccount},
    ports::{secrets, secrets::Secrets},
    usecases::processor::process,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct Env {
    splitwise_api_key: String,
    ynab_api_key: String,
    ynab_budget_id: String,
    ynab_splitwise_account_id: String,
    ynab_splitwise_transfer_id: String,
    ynab_expenses_account_id: String,
    ynab_expenses_transfer_id: String,
    splitwise_user_id: i64,
}

impl FromStr for Env {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        from_str(s)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    HookBuilder::new().theme(Theme::new()).install()?;

    let sdk_config = aws_config::from_env()
        .region(RegionProviderChain::default_provider().or_else("ap-southeast-2"))
        .load()
        .await;

    let secrets = secrets::combine(env::Env, AWSSecretsManager::new(&sdk_config));
    let secrets_manager_key = secrets.get::<String>("AWS_SECRETS_MANAGER_ARN").await?;

    let env = secrets.get::<Env>(&secrets_manager_key).await?;

    let expense_tracker = &splitwise::new(env.splitwise_api_key);
    let budget = &Ynab::new(env.ynab_budget_id, env.ynab_api_key);

    let records = &DynamoDB::new(&sdk_config, "splitwise");

    let config = &Config {
        splitwise: YNABAccount {
            account_id: env.ynab_splitwise_account_id,
            transfer_id: env.ynab_splitwise_transfer_id,
        },
        expenses: YNABAccount {
            account_id: env.ynab_expenses_account_id,
            transfer_id: env.ynab_expenses_transfer_id,
        },
        splitwise_user_id: env.splitwise_user_id,
    };

    let handler = service_fn(
        async move |_: LambdaEvent<serde_json::Value>| -> Result<(), String> {
            process(config, expense_tracker, budget, records)
                .await
                .map_err(|report| format!("{:?}", report))
        },
    );

    run(handler)
        .await
        .map_err(|e| eyre!(e))
        .wrap_err("Error running handler")
}
