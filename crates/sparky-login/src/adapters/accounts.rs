use nix::unistd::User;

use crate::domain::environment::Account;
use crate::ports::accounts::Accounts;

pub struct SystemAccounts;

impl Accounts for SystemAccounts {
    fn find(&self, name: &str) -> Option<Account> {
        let user = User::from_name(name).ok().flatten()?;
        Some(Account {
            name: user.name,
            uid: user.uid.as_raw(),
            gid: user.gid.as_raw(),
            home: user.dir,
            shell: user.shell,
        })
    }
}

#[cfg(test)]
mod tests {
    use nix::unistd::{User, getuid};

    use super::SystemAccounts;
    use crate::ports::accounts::Accounts;

    #[test]
    fn acha_a_conta_pelo_nome_com_uid_e_pasta() {
        let Some(eu) = User::from_uid(getuid()).ok().flatten() else {
            panic!("o usuário dos testes deveria existir");
        };
        let Some(conta) = SystemAccounts.find(&eu.name) else {
            panic!("deveria achar {}", eu.name);
        };
        assert_eq!((conta.uid, conta.home), (getuid().as_raw(), eu.dir));
    }

    #[test]
    fn nome_desconhecido_nao_tem_conta() {
        assert!(SystemAccounts.find("sparky-ninguem-por-aqui").is_none());
    }
}
