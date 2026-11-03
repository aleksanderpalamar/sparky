use std::io::{self, BufReader};
use std::process::ExitCode;

use sparky_login::adapters::accounts::SystemAccounts;
use sparky_login::adapters::appearance::UserAppearance;
use sparky_login::adapters::launcher::UserLauncher;
use sparky_login::adapters::pam::LinuxPam;
use sparky_login::adapters::stop;
use sparky_login::cli::{self, Command};
use sparky_login::service::{self, settings::Settings};
use sparky_login::worker::Worker;

const USAGE: u8 = 2;

fn main() -> ExitCode {
    let Some(command) = cli::parse(std::env::args().skip(1)) else {
        eprintln!("{}", cli::USAGE);
        return ExitCode::from(USAGE);
    };
    if !nix::unistd::geteuid().is_root() {
        eprintln!("sparky-login: precisa rodar como root, pelo serviço sparky-login");
        return ExitCode::FAILURE;
    }
    let result = match command {
        Command::Worker => work(),
        Command::Daemon { vt } => serve(vt),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-login: {error}");
            ExitCode::FAILURE
        }
    }
}

fn work() -> Result<(), String> {
    stop::watch()?;
    let pam = LinuxPam::default();
    let worker = Worker {
        pam: &pam,
        accounts: &SystemAccounts,
        launcher: &UserLauncher,
        appearance: &UserAppearance,
    };
    let mut input = BufReader::new(io::stdin().lock());
    worker.run(&mut input, &mut io::stdout().lock())
}

fn serve(vt: u32) -> Result<(), String> {
    let worker =
        std::env::current_exe().map_err(|error| format!("caminho do sparky-login: {error}"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    runtime.block_on(service::serve(Settings::new(vt, worker)))
}
