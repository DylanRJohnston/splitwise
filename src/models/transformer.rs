use super::{Cents, Expense, Share, Transaction};
use splitwise::own;

#[derive(Debug)]
pub struct YNABAccount {
    pub id: String,
    pub name: String,
    pub transfer_id: String,
}

#[derive(Debug)]
pub struct Config {
    pub splitwise_account: YNABAccount,
    pub expenses_account: YNABAccount,
    pub splitwise_user_id: i64,
}

fn payment(account_id: &str, transfer_id: &str, expense: &Expense) -> Vec<Transaction> {
    vec![own!(Transaction {
        account_id: account_id,
        date: expense.date,
        amount: -1 * expense.cost.milli_dollars(),
        payee_id: Some(transfer_id),
        payee_name: None,
        memo: format!("{} settling up", expense.created_by.full_name()),
        cleared: format!("cleared"),
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
            amount: -1 * your_share.paid_share.milli_dollars(),
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
            amount: -1 * share.net_balance.milli_dollars(),
            payee_id: None,
            payee_name: Some(share.user.full_name()),
            memo: expense.description,
            cleared: "cleared",
            approved: false,
        }))
    }

    transactions
}

pub type Transformer = impl Fn(&[Expense]) -> Vec<Transaction>;

pub fn new(config: Config) -> Transformer {
    move |expenses| {
        expenses
            .into_iter()
            .flat_map(|expense| {
                if !expense.payment {
                    return shared_expense(
                        config.splitwise_user_id,
                        &config.expenses_account.id,
                        &config.splitwise_account.id,
                        expense,
                    );
                }

                if expense.created_by.id == config.splitwise_user_id {
                    return payment(
                        &config.expenses_account.id,
                        &config.splitwise_account.transfer_id,
                        expense,
                    );
                }

                return payment(
                    &config.splitwise_account.id,
                    &config.expenses_account.transfer_id,
                    expense,
                );
            })
            .collect()
    }
}
