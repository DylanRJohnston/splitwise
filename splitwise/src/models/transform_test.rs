use anyhow::Result;
use own::own;
use pretty_assertions::assert_eq;
use std::{fs, path::Path};
use transformer::YNABAccount;

use crate::models::transformer;

use super::{Expense, Expenses, Transaction, Transformer};

fn load_test_data() -> Result<Vec<Expense>> {
    let path = Path::new("./tests/data.json").canonicalize()?;
    let raw = fs::read_to_string(path)?;

    Ok(serde_json::from_str::<Expenses>(&raw)?.expenses)
}

fn new_transformer() -> Transformer {
    transformer::new(own!(transformer::Config {
        splitwise: YNABAccount {
            account_id: "splitwise-account-id",
            transfer_id: "splitwise-transfer-id",
        },
        expenses: YNABAccount {
            account_id: "expenses-account-id",
            transfer_id: "expenses-account-transfer-id",
        },
        splitwise_user_id: 2,
    }))
}

#[test]
pub fn test_direct_expense() -> Result<()> {
    let expenses = &load_test_data()?[0..1];

    let transactions = new_transformer()(expenses);

    assert_eq!(transactions.len(), 2);

    assert_eq!(
        transactions[0],
        own!(Transaction {
            account_id: "expenses-account-id",
            date: expenses[0].date,
            amount: (-83250),
            payee_id: None,
            payee_name: None,
            memo: "Groceries",
            cleared: "uncleared",
            approved: false,
        }),
    );

    assert_eq!(
        transactions[1],
        own!(Transaction {
            account_id: "splitwise-account-id",
            date: expenses[0].date,
            amount: 41630,
            payee_id: None,
            payee_name: Some("Bar Bar"),
            memo: "Groceries",
            cleared: "cleared",
            approved: false,
        }),
    );

    Ok(())
}

#[test]
pub fn test_indirect_expense() -> Result<()> {
    let expenses = &load_test_data()?[2..3];

    let transactions = new_transformer()(expenses);

    assert_eq!(transactions.len(), 1);

    assert_eq!(
        transactions[0],
        own!(Transaction {
            account_id: "splitwise-account-id",
            date: expenses[0].date,
            amount: (-121820),
            payee_id: None,
            payee_name: Some("Bar Bar"),
            memo: "Gas",
            cleared: "cleared",
            approved: false,
        })
    );

    Ok(())
}

#[test]
pub fn test_payment_to_you() -> Result<()> {
    let expenses = &load_test_data()?[1..2];

    let transactions = new_transformer()(expenses);

    assert_eq!(transactions.len(), 1);

    assert_eq!(
        transactions[0],
        own!(Transaction {
            account_id: "splitwise-account-id",
            date: expenses[0].date,
            amount: (-87870),
            payee_id: Some("expenses-account-transfer-id"),
            payee_name: None,
            memo: "Bar Bar settling up",
            cleared: "cleared",
            approved: false,
        })
    );

    Ok(())
}

#[test]
pub fn payment_from_you() -> Result<()> {
    let expenses = &load_test_data()?[3..4];

    let transactions = new_transformer()(expenses);

    assert_eq!(transactions.len(), 1);

    assert_eq!(
        transactions[0],
        own!(Transaction {
            account_id: "expenses-account-id",
            date: expenses[0].date,
            amount: (-81690),
            payee_id: Some("splitwise-transfer-id"),
            payee_name: None,
            memo: "Foo Foo settling up",
            cleared: "cleared",
            approved: false,
        })
    );

    Ok(())
}
