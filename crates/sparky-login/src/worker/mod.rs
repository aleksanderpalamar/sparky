#[cfg(test)]
mod fakes;
#[cfg(test)]
mod scene;
#[cfg(test)]
mod worker_greeter_tests;
#[cfg(test)]
mod worker_tests;

use std::io::{BufRead, Write};
use std::path::Path;

use sparky_login_core::codec::{receive, send};
use sparky_login_core::protocol::{Refusal, SOCKET_VARIABLE};
use sparky_login_core::sessions::Session;

use crate::domain::environment::{Account, Class, environment, login_command, pam_variables, tty};
use crate::domain::job::{Job, LoginJob, Order, Report};
use crate::ports::accounts::Accounts;
use crate::ports::appearance::Appearance;
use crate::ports::launcher::Launcher;
use sparky_pam::{Pam, Transaction};

pub const LOGIN_SERVICE: &str = "sparky-login";
pub const GREETER_SERVICE: &str = "sparky-login-greeter";

pub struct Worker<'a> {
    pub pam: &'a dyn Pam,
    pub accounts: &'a dyn Accounts,
    pub launcher: &'a dyn Launcher,
    pub appearance: &'a dyn Appearance,
    pub locale: &'a [(String, String)],
}

impl Worker<'_> {
    pub fn run(&self, input: &mut impl BufRead, output: &mut impl Write) -> Result<(), String> {
        let job = receive::<Job>(input).map_err(|error| error.to_string())?;
        match job.ok_or("o daemon não mandou tarefa")? {
            Job::Greeter {
                user,
                command,
                vt,
                socket,
            } => self.greeter(&user, &command, vt, &socket, output),
            Job::Login(job) => self.login(job, input, output),
        }
    }

    fn greeter(
        &self,
        user: &str,
        command: &[String],
        vt: u32,
        socket: &Path,
        output: &mut impl Write,
    ) -> Result<(), String> {
        let account = self
            .accounts
            .find(user)
            .ok_or_else(|| format!("o usuário {user} não existe"))?;
        let mut transaction = self.pam.begin(GREETER_SERVICE, user, None)?;
        open(transaction.as_mut(), Class::Greeter, vt, None)?;
        let mut variables = environment(&account, transaction.environment());
        variables.extend(self.locale.iter().cloned());
        variables.push((
            SOCKET_VARIABLE.to_owned(),
            socket.to_string_lossy().into_owned(),
        ));
        let result = self.run_session(&account, command, &variables, vt, output);
        finish(transaction, output, result)
    }

    fn login(
        &self,
        job: LoginJob,
        input: &mut impl BufRead,
        output: &mut impl Write,
    ) -> Result<(), String> {
        let LoginJob {
            username,
            password,
            session,
            vt,
            appearance,
        } = job;
        let username = username.as_str();
        let session = &session;
        let mut transaction = self.pam.begin(LOGIN_SERVICE, username, Some(password))?;
        if let Err(refusal) = transaction
            .authenticate()
            .and_then(|()| transaction.authorize())
        {
            return report(output, Report::Refused { refusal });
        }
        let Some(account) = self.accounts.find(username) else {
            let refusal = Refusal::Unavailable(format!(
                "a conta {username} não tem registro neste computador"
            ));
            return report(output, Report::Refused { refusal });
        };
        report(output, Report::Authenticated)?;
        if !matches!(receive::<Order>(input), Ok(Some(Order::Start))) {
            return Ok(());
        }
        if let Err(message) = open(transaction.as_mut(), Class::User, vt, Some(session)) {
            return report(output, Report::Failed { message });
        }
        let variables = environment(&account, transaction.environment());
        let command = login_command(&account, &session.exec);
        self.appearance.capture(&account, &variables, &appearance);
        let result = self.run_session(&account, &command, &variables, vt, output);
        self.appearance.capture(&account, &variables, &appearance);
        finish(transaction, output, result)
    }

    fn run_session(
        &self,
        account: &Account,
        command: &[String],
        variables: &[(String, String)],
        vt: u32,
        output: &mut impl Write,
    ) -> Result<(), String> {
        let mut process = self
            .launcher
            .launch(account, command, variables, &tty(vt))?;
        report(output, Report::Running)?;
        process.wait()
    }
}

fn finish(
    mut transaction: Box<dyn Transaction>,
    output: &mut impl Write,
    result: Result<(), String>,
) -> Result<(), String> {
    transaction.close();
    report(output, Report::Ended)?;
    result
}

fn open(
    transaction: &mut dyn Transaction,
    class: Class,
    vt: u32,
    session: Option<&Session>,
) -> Result<(), String> {
    transaction.set_tty(&tty(vt).to_string_lossy())?;
    for (name, value) in pam_variables(class, vt, session) {
        transaction.put(&name, &value)?;
    }
    transaction.establish()?;
    transaction.open()
}

fn report(output: &mut impl Write, report: Report) -> Result<(), String> {
    send(output, &report).map_err(|error| error.to_string())
}
