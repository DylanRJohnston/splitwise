use anyhow::{Ok, Result};

use crate::{
    models::transformer,
    ports::{budget::Budget, expense_tracker::ExpenseTracker, store::Set},
};

pub fn process(
    config: transformer::Config,
    expense_tracker: impl ExpenseTracker,
    budget: impl Budget,
    state: impl Set,
) -> Result<()> {
    let to_transaction = transformer::new(config);

    let new_expenses = expense_tracker
        .get_all_expenses()?
        .expenses
        .into_iter()
        .try_fold(vec![], |mut acc, it| {
            if !state.has(it.id)? {
                acc.push(it);
            }
            Ok(acc)
        })?;

    let expense_ids = new_expenses.iter().map(|it| it.id).collect::<Vec<i64>>();

    budget.create_transactions(to_transaction(&new_expenses))?;
    state.batch_add(expense_ids)?;

    Ok(())
}
