use crate::domain::environment::Account;

pub trait Accounts {
    fn find(&self, name: &str) -> Option<Account>;
}
