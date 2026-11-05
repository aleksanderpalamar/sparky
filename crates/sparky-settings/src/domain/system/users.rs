use crate::domain::system::passwd::{self, Account};
use crate::ports::commands::CommandRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub account: Account,
    pub groups: Option<Vec<String>>,
    pub logged_in: Option<bool>,
}

impl User {
    pub fn title(&self) -> String {
        match self.account.full_name.is_empty() {
            true => self.account.name.clone(),
            false => format!("{} ({})", self.account.full_name, self.account.name),
        }
    }

    pub fn role(&self) -> String {
        let Some(groups) = &self.groups else {
            return crate::i18n::text("unavailable");
        };
        let key = match groups
            .iter()
            .any(|group| matches!(group.as_str(), "wheel" | "sudo"))
        {
            true => "users-administrator",
            false => "users-regular",
        };
        crate::i18n::text(key)
    }

    pub fn session(&self) -> String {
        let key = match self.logged_in {
            Some(true) => "users-session-open",
            Some(false) => "users-session-closed",
            None => "unavailable",
        };
        crate::i18n::text(key)
    }
}

pub fn load(commands: &dyn CommandRunner) -> Vec<User> {
    let passwd = commands.run("getent", &["passwd"]).unwrap_or_default();
    let groups = commands.run("getent", &["group"]).ok();
    let sessions = commands
        .run("loginctl", &["list-users", "--no-legend"])
        .ok();
    let active: Option<Vec<_>> = sessions.as_ref().map(|output| {
        output
            .lines()
            .filter_map(|line| line.split_whitespace().nth(1))
            .collect()
    });
    passwd::humans(&passwd)
        .into_iter()
        .map(|account| User {
            groups: groups
                .as_ref()
                .map(|contents| passwd::groups_of(contents, &account)),
            logged_in: active
                .as_ref()
                .map(|names| names.contains(&account.name.as_str())),
            account,
        })
        .collect()
}
