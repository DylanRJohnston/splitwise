use super::{Cents, Expense, Share, Transaction};
use own::own;

#[derive(Debug, Clone)]
pub struct YNABAccount {
    pub account_id: String,
    pub transfer_id: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub splitwise: YNABAccount,
    pub expenses: YNABAccount,
    pub splitwise_user_id: i64,
}

#[allow(clippy::redundant_clone)]
fn payment(account_id: &str, transfer_id: &str, expense: &Expense) -> Vec<Transaction> {
    vec![own!(Transaction {
        account_id: account_id,
        date: expense.date,
        amount: -expense.cost.milli_dollars(),
        payee_id: Some(transfer_id),
        payee_name: None,
        memo: format!("{} settling up", expense.created_by.full_name()),
        cleared: "cleared",
        approved: false,
    })]
}

fn shared_expense(
    user_id: i64,
    expenses_account_id: &str,
    splitwise_account_id: &str,
    expense: &Expense,
) -> Vec<Transaction> {
    let your_share = expense.users.iter().find(|share| share.user_id == user_id);

    let other_shares = expense
        .users
        .iter()
        .filter(|share| share.user_id != user_id)
        .collect::<Vec<&Share>>();

    let mut transactions: Vec<Transaction> = Vec::new();

    if let Some(your_share) = your_share && your_share.paid_share > Cents(0) {
        transactions.push(own!(Transaction {
            account_id: expenses_account_id,
            date: expense.date,
            amount: -your_share.paid_share.milli_dollars(),
            payee_id: None,
            payee_name: None,
            memo: expense.description,
            cleared: "uncleared",
            approved: false,
        }))
    }

    for share in other_shares {
        transactions.push(own!(Transaction {
            account_id: splitwise_account_id,
            date: expense.date,
            amount: -share.net_balance.milli_dollars(),
            payee_id: None,
            payee_name: Some(share.user.full_name()),
            memo: expense.description,
            cleared: "cleared",
            approved: false,
        }))
    }

    transactions
}

enum Kind {
    SharedExpense,
    SettleUp,
    Payment,
}

fn classify(user_id: i64, expense: &Expense) -> Kind {
    if !expense.payment {
        Kind::SharedExpense
    } else if expense.created_by.id == user_id {
        Kind::SettleUp
    } else {
        Kind::Payment
    }
}

pub type Transformer<'a> = impl Fn(&[Expense]) -> Vec<Transaction> + 'a;

pub fn new(config: &Config) -> Transformer {
    move |expenses: &[Expense]| {
        expenses
            .iter()
            .flat_map(
                |expense| match classify(config.splitwise_user_id, expense) {
                    Kind::SharedExpense => shared_expense(
                        config.splitwise_user_id,
                        &config.expenses.account_id,
                        &config.splitwise.account_id,
                        expense,
                    ),
                    Kind::SettleUp => payment(
                        &config.expenses.account_id,
                        &config.splitwise.transfer_id,
                        expense,
                    ),
                    Kind::Payment => payment(
                        &config.splitwise.account_id,
                        &config.expenses.transfer_id,
                        expense,
                    ),
                },
            )
            .collect()
    }
}
