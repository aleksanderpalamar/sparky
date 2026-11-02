pub mod link;
pub mod runtime;
pub mod settings;
pub mod watch;
pub mod workers;

use sparky_login_core::session_files;
use sparky_login_core::sessions::{Session, sessions};
use sparky_xdg::desktop_entry::locale_from_env;
use sparky_xdg::programs::installed;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc::unbounded_channel;

use crate::adapters::accounts::SystemAccounts;
use crate::adapters::{memory_file, terminal};
use crate::domain::machine::{Effect, Event, Machine};
use crate::ports::accounts::Accounts;
use crate::service::link::{Input, bind, listen};
use crate::service::runtime::Runtime;
use crate::service::settings::Settings;

pub async fn serve(settings: Settings) -> Result<(), String> {
    let greeter = SystemAccounts.find(&settings.greeter_user).ok_or_else(|| {
        format!(
            "o usuário {} não existe; rode systemd-sysusers",
            settings.greeter_user
        )
    })?;
    if let Err(error) = terminal::activate(settings.vt) {
        eprintln!(
            "sparky-login: não deu para ir ao terminal {}: {error}",
            settings.vt
        );
    }
    let listener = bind(&settings.socket, greeter.uid)?;
    let mut terminate = signal(SignalKind::terminate()).map_err(|error| error.to_string())?;
    let mut interrupt = signal(SignalKind::interrupt()).map_err(|error| error.to_string())?;
    let (inputs, mut received) = unbounded_channel();
    tokio::spawn(listen(listener, greeter.uid, inputs.clone()));
    let mut machine = Machine::new(discover(&settings), memory_file::load(&settings.memory));
    let mut runtime = Runtime::new(settings, inputs.clone());
    let _ = inputs.send(Input::Event(Event::Started));
    loop {
        tokio::select! {
            Some(input) = received.recv() => step(&mut machine, &mut runtime, input).await,
            _ = terminate.recv() => break,
            _ = interrupt.recv() => break,
        }
    }
    runtime.shutdown();
    Ok(())
}

async fn step(machine: &mut Machine, runtime: &mut Runtime, input: Input) {
    let event = match input {
        Input::Connected(writer) => {
            runtime.connection = Some(writer);
            return;
        }
        Input::Event(event) => event,
        Input::Login { attempt, event } => {
            if !runtime.current(attempt) {
                return;
            }
            if matches!(event, Event::SessionEnded) {
                runtime.session_ended();
            }
            event
        }
    };
    for effect in machine.on(event) {
        if effect == Effect::StartGreeter {
            machine.set_sessions(discover(runtime.settings()));
        }
        runtime.apply(effect).await;
    }
}

fn discover(settings: &Settings) -> Vec<Session> {
    let folders = session_files::folders(settings.data_dirs.as_deref());
    sessions(
        session_files::from_folders(&folders),
        &locale_from_env(),
        installed,
    )
}
