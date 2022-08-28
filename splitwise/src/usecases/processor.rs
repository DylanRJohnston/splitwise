use crate::{
    models::{transformer, Record, ID},
    ports::{budget::Budget, expense_tracker::ExpenseTracker, store::Store},
};
use color_eyre::Result;
use tracing::instrument;

// We have to use &impl so that the lambda runtime wrapper is FnMut and not FnOnce
// `move` closures are `FnOnce` if they consume the values they move
// async closures with arguments must be `move`
#[instrument]
pub async fn process(
    config: &transformer::Config,
    expense_tracker: &impl ExpenseTracker,
    budget: &impl Budget,
    records: &impl Store,
) -> Result<()> {
    let to_transaction = transformer::new(config);

    let all_expenses = expense_tracker.get_all_expenses().await?.expenses;
    let expense_ids = all_expenses.iter().map(ID::id).collect::<Vec<_>>();

    println!("Expenses from splitwise: {}", expense_ids.len());

    let already_processed = records.batch_has(&expense_ids).await?;

    let new_expenses = all_expenses
        .into_iter()
        .filter(|it| !already_processed.contains(&it.id()))
        .collect::<Vec<_>>();

    println!("New expenses: {}", new_expenses.len());

    budget
        .create_transactions(to_transaction(&new_expenses))
        .await?;

    println!("Successfully saved to YNAB");

    records
        .batch_add(
            &new_expenses
                .iter()
                .map(Into::<Record>::into)
                .collect::<Vec<_>>(),
        )
        .await?;

    println!("Successfully saved to DynamoDB");

    Ok(())
}
