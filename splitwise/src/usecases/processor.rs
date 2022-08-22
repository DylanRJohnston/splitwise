use anyhow::{Ok, Result};

use crate::{
    models::{transformer, Record, ID},
    ports::{budget::Budget, expense_tracker::ExpenseTracker, store::Store},
};

pub async fn process(
    config: transformer::Config,
    expense_tracker: impl ExpenseTracker,
    budget: impl Budget,
    records: impl Store,
) -> Result<()> {
    let to_transaction = transformer::new(config);

    let all_expenses = expense_tracker.get_all_expenses().await?.expenses;
    let expense_ids = all_expenses.iter().map(ID::id).collect::<Vec<_>>();
    let already_processed = records.batch_has(&expense_ids).await?;

    let new_expenses = all_expenses
        .into_iter()
        .filter(|it| !already_processed.contains(&it.id()))
        .collect::<Vec<_>>();

    budget
        .create_transactions(to_transaction(&new_expenses))
        .await?;

    records
        .batch_add(
            &new_expenses
                .iter()
                .map(Into::<Record>::into)
                .collect::<Vec<_>>(),
        )
        .await?;

    Ok(())
}
