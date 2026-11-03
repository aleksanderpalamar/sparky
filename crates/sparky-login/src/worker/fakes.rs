use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use sparky_login_core::protocol::Refusal;
use zeroize::Zeroizing;

use crate::domain::environment::Account;
use crate::ports::accounts::Accounts;
use crate::ports::appearance::Appearance;
use crate::ports::launcher::{Launcher, Process};
use sparky_pam::{Pam, Transaction};

pub type Log = Rc<RefCell<Vec<String>>>;

pub struct FakePam {
    pub log: Log,
    pub verdict: Result<(), Refusal>,
}

struct FakeTransaction {
    log: Log,
    verdict: Result<(), Refusal>,
    variables: Vec<(String, String)>,
}

impl Pam for FakePam {
    fn begin(
        &self,
        service: &str,
        user: &str,
        password: Option<Zeroizing<String>>,
    ) -> Result<Box<dyn Transaction>, String> {
        let secret = password.map_or_else(|| "-".to_owned(), |password| password.to_string());
        self.log
            .borrow_mut()
            .push(format!("begin {service} {user} {secret}"));
        Ok(Box::new(FakeTransaction {
            log: Rc::clone(&self.log),
            verdict: self.verdict.clone(),
            variables: Vec::new(),
        }))
    }
}

impl Transaction for FakeTransaction {
    fn authenticate(&mut self) -> Result<(), Refusal> {
        self.log.borrow_mut().push("authenticate".into());
        self.verdict.clone()
    }

    fn set_tty(&mut self, tty: &str) -> Result<(), String> {
        self.log.borrow_mut().push(format!("tty {tty}"));
        Ok(())
    }

    fn put(&mut self, name: &str, value: &str) -> Result<(), String> {
        self.variables.push((name.to_owned(), value.to_owned()));
        Ok(())
    }

    fn establish(&mut self) -> Result<(), String> {
        self.log.borrow_mut().push("establish".into());
        Ok(())
    }

    fn open(&mut self) -> Result<(), String> {
        self.log.borrow_mut().push("open".into());
        Ok(())
    }

    fn environment(&self) -> Vec<(String, String)> {
        let mut variables = self.variables.clone();
        variables.push(("XDG_RUNTIME_DIR".into(), "/run/user/1000".into()));
        variables
    }

    fn close(&mut self) {
        self.log.borrow_mut().push("close".into());
    }
}

pub struct FakeAccounts;

impl Accounts for FakeAccounts {
    fn find(&self, name: &str) -> Option<Account> {
        let home = PathBuf::from(format!("/home/{name}"));
        let shell = PathBuf::from("/usr/bin/zsh");
        (name != "fantasma").then(|| Account {
            name: name.to_owned(),
            uid: 1000,
            gid: 1000,
            home,
            shell,
        })
    }
}

pub struct Launched {
    pub command: Vec<String>,
    pub environment: Vec<(String, String)>,
    pub tty: PathBuf,
}

pub struct FakeLauncher {
    pub log: Log,
    pub launched: RefCell<Vec<Launched>>,
}

struct FakeProcess {
    log: Log,
}

impl Launcher for FakeLauncher {
    fn launch(
        &self,
        account: &Account,
        command: &[String],
        environment: &[(String, String)],
        tty: &Path,
    ) -> Result<Box<dyn Process>, String> {
        self.log
            .borrow_mut()
            .push(format!("launch {}", account.name));
        self.launched.borrow_mut().push(Launched {
            command: command.to_vec(),
            environment: environment.to_vec(),
            tty: tty.to_path_buf(),
        });
        Ok(Box::new(FakeProcess {
            log: Rc::clone(&self.log),
        }))
    }
}

impl Process for FakeProcess {
    fn wait(&mut self) -> Result<(), String> {
        self.log.borrow_mut().push("wait".into());
        Ok(())
    }
}

pub struct FakeAppearance {
    pub log: Log,
}

impl Appearance for FakeAppearance {
    fn capture(&self, account: &Account, _environment: &[(String, String)], folder: &Path) {
        self.log
            .borrow_mut()
            .push(format!("capture {} {}", account.name, folder.display()));
    }
}
