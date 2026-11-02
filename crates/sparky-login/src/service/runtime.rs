use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use sparky_login_core::protocol::{Power, Refusal, Reply};
use tokio::net::unix::OwnedWriteHalf;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::adapters::{logind, memory_file};
use crate::domain::job::{Job, Order};
use crate::domain::machine::{Effect, Event, Login};
use crate::service::link::Input;
use crate::service::settings::Settings;
use crate::service::watch::{greeter, login};
use crate::service::workers::{send, spawn};

const COOLDOWN: Duration = Duration::from_secs(2);

struct Session {
    attempt: u64,
    writer: OwnedWriteHalf,
    pid: Option<u32>,
    started: bool,
}

pub struct Runtime {
    settings: Settings,
    inputs: UnboundedSender<Input>,
    pub connection: Option<OwnedWriteHalf>,
    greeter_stop: Option<oneshot::Sender<()>>,
    greeter_pid: Option<u32>,
    session: Option<Session>,
    attempts: u64,
    last_greeter: Option<Instant>,
}

impl Runtime {
    pub fn new(settings: Settings, inputs: UnboundedSender<Input>) -> Self {
        Self {
            settings,
            inputs,
            connection: None,
            greeter_stop: None,
            greeter_pid: None,
            session: None,
            attempts: 0,
            last_greeter: None,
        }
    }

    pub async fn apply(&mut self, effect: Effect) {
        match effect {
            Effect::StartGreeter => self.start_greeter().await,
            Effect::Reply(reply) => self.reply(&reply).await,
            Effect::Authenticate(login) => self.authenticate(login).await,
            Effect::Power(action) => self.power(action),
            Effect::Remember(memory) => {
                if let Err(error) = memory_file::save(&self.settings.memory, &memory) {
                    eprintln!("sparky-login: {}: {error}", self.settings.memory.display());
                }
            }
            Effect::StopGreeter => {
                if let Some(stop) = self.greeter_stop.take() {
                    let _ = stop.send(());
                }
            }
            Effect::StartSession => self.start_session().await,
            Effect::Abandon => self.abandon(),
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn current(&self, attempt: u64) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.attempt == attempt)
    }

    pub fn session_ended(&mut self) {
        self.session = None;
    }

    pub fn shutdown(&mut self) {
        terminate(self.greeter_pid.take());
        if self
            .session
            .as_ref()
            .is_some_and(|session| !session.started)
        {
            self.abandon();
        }
    }

    async fn start_greeter(&mut self) {
        if self
            .last_greeter
            .is_some_and(|last| last.elapsed() < COOLDOWN)
        {
            tokio::time::sleep(COOLDOWN).await;
        }
        self.last_greeter = Some(Instant::now());
        self.connection = None;
        let settings = &self.settings;
        let job = Job::Greeter {
            user: settings.greeter_user.clone(),
            command: settings.greeter_command.clone(),
            vt: settings.vt,
            socket: settings.socket.clone(),
        };
        match spawn(&settings.worker, &job).await {
            Ok(link) => {
                let (stop, stopped) = oneshot::channel();
                self.greeter_stop = Some(stop);
                self.greeter_pid = link.child.id();
                tokio::spawn(greeter(link, stopped, self.inputs.clone()));
            }
            Err(error) => {
                eprintln!("sparky-login: a tela de login não abriu: {error}");
                let _ = self.inputs.send(Input::Event(Event::GreeterGone));
            }
        }
    }

    async fn reply(&mut self, reply: &Reply) {
        let Some(writer) = self.connection.as_mut() else {
            return;
        };
        if send(writer, reply).await.is_err() {
            self.connection = None;
        }
    }

    async fn authenticate(&mut self, login_request: Login) {
        let Login {
            username,
            password,
            session,
        } = login_request;
        let job = Job::Login {
            username,
            password,
            session,
            vt: self.settings.vt,
        };
        match spawn(&self.settings.worker, &job).await {
            Ok(link) => {
                self.attempts += 1;
                let attempt = self.attempts;
                let pid = link.child.id();
                self.session = Some(Session {
                    attempt,
                    writer: link.writer,
                    pid,
                    started: false,
                });
                tokio::spawn(login(link.child, link.reader, self.inputs.clone(), attempt));
            }
            Err(error) => {
                let _ = self
                    .inputs
                    .send(Input::Event(Event::Refused(Refusal::Unavailable(error))));
            }
        }
    }

    fn power(&self, action: Power) {
        let inputs = self.inputs.clone();
        tokio::spawn(async move {
            let result = logind::power(action).await;
            let _ = inputs.send(Input::Event(Event::Powered(result)));
        });
    }

    async fn start_session(&mut self) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        session.started = true;
        if let Err(error) = send(&mut session.writer, &Order::Start).await {
            eprintln!("sparky-login: a sessão não recebeu a ordem de abrir: {error}");
        }
    }

    fn abandon(&mut self) {
        if let Some(session) = self.session.take() {
            terminate(session.pid);
        }
    }
}

fn terminate(pid: Option<u32>) {
    let Some(pid) = pid.and_then(|pid| i32::try_from(pid).ok()) else {
        return;
    };
    let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
}
