use anyhow::{Ok, Result};

use crate::{
    models::{transformer, Record},
    ports::{budget::Budget, expense_tracker::ExpenseTracker, store::Set},
};

pub async fn process(
    config: transformer::Config,
    expense_tracker: impl ExpenseTracker,
    budget: impl Budget,
    records: impl Set<Record>,
) -> Result<()> {
    let to_transaction = transformer::new(config);

    let all_expenses = expense_tracker.get_all_expenses().await?.expenses;
    let old_expenses = records
        .batch_has(&all_expenses.iter().map(Into::into).collect::<Vec<_>>())
        .await?;

    let new_expenses = all_expenses
        .into_iter()
        .filter(|it| !old_expenses.contains_key(&it.id))
        .collect::<Vec<_>>();

    budget
        .create_transactions(to_transaction(&new_expenses))
        .await?;

    records
        .batch_add(&new_expenses.iter().map(Into::into).collect::<Vec<_>>())
        .await?;

    Ok(())
}
