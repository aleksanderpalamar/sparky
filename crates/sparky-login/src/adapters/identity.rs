use std::ffi::CString;

use nix::unistd::{Gid, getgrouplist};

use crate::domain::environment::Account;

pub fn groups(account: &Account) -> Result<Vec<Gid>, String> {
    let name = CString::new(account.name.as_str()).map_err(|error| error.to_string())?;
    getgrouplist(&name, Gid::from_raw(account.gid))
        .map_err(|error| format!("grupos de {}: {error}", account.name))
}
