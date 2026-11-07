use std::env;
use std::io::{self, ErrorKind, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use crate::config::cli::{CliError, Command, ConfigSource, parse};
use crate::config::load::{LoadError, load};
use crate::config::path::config_path;
use crate::config::settings::{Partial, Settings};
use crate::domain::field::Field;
use crate::render::view::View;
use crate::{i18n, render, run, sources, terminal};

const USAGE: u8 = 2;

enum Failure {
    Usage(CliError),
    Config(LoadError),
    Output(String),
}

impl Failure {
    fn code(&self) -> ExitCode {
        match self {
            Failure::Usage(_) => ExitCode::from(USAGE),
            Failure::Config(_) | Failure::Output(_) => ExitCode::FAILURE,
        }
    }

    fn message(&self) -> String {
        match self {
            Failure::Usage(error) => format!("{error}\n{}", i18n::text("usage")),
            Failure::Config(error) => error.to_string(),
            Failure::Output(error) => error.clone(),
        }
    }
}

pub fn run(arguments: &[String]) -> ExitCode {
    match execute(arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("sparky-fetch: {}", failure.message());
            failure.code()
        }
    }
}

fn execute(arguments: &[String]) -> Result<(), Failure> {
    match parse(arguments).map_err(Failure::Usage)? {
        Command::Help => emit(&format!("{}\n", i18n::text("usage"))),
        Command::Version => emit(&format!("sparky-fetch {}\n", env!("CARGO_PKG_VERSION"))),
        Command::ListFields => emit(&fields()),
        Command::Run { partial, config } => fetch(partial, &config),
    }
}

fn fetch(partial: Partial, config: &ConfigSource) -> Result<(), Failure> {
    let file = load(config, &default_path()).map_err(Failure::Config)?;
    let partials: Vec<Partial> = file.into_iter().chain(std::iter::once(partial)).collect();
    let settings = Settings::default().merged(&partials);
    let report = run::collect(sources::all(&settings), &settings.fields, settings.limits());
    let view = View::resolve(&settings, &terminal::detect());
    let text = render::render(&report, &view).map_err(Failure::Output)?;
    emit(&text)
}

fn default_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    let config_home = env::var("XDG_CONFIG_HOME").ok();
    config_path(config_home.as_deref(), &home)
}

fn fields() -> String {
    Field::ALL
        .iter()
        .map(|field| format!("{}\t{}\n", field.key(), render::label(*field)))
        .collect()
}

fn emit(text: &str) -> Result<(), Failure> {
    let mut stdout = io::stdout().lock();
    match stdout
        .write_all(text.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(Failure::Output(error.to_string())),
    }
}
