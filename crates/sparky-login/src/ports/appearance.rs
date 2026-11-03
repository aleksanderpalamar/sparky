use std::path::Path;

use crate::domain::environment::Account;

pub trait Appearance {
    fn capture(&self, account: &Account, environment: &[(String, String)], folder: &Path);
}
