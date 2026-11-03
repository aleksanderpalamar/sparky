use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

use sparky_ipc::{Client, Event as Notice, IdleHold, Reply, Request, SessionAction};
use tokio::sync::mpsc::Sender;

use crate::app::Event;
use crate::domain::guard::Input;
use crate::domain::inhibitions::Hold;
use crate::ports::Compositor;

const UNEXPECTED: &str = "resposta inesperada do sparky";
const LOST: &str = "a conexão da tela acesa ficou inacessível";

pub struct Sparky {
    path: PathBuf,
    link: Mutex<Option<Client>>,
}

impl Sparky {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            link: Mutex::new(None),
        }
    }

    fn session(&self, action: SessionAction) -> Result<(), String> {
        self.ask(&Request::Session { action })
    }

    fn ask(&self, request: &Request) -> Result<(), String> {
        let mut client = Client::connect(&self.path).map_err(|error| error.to_string())?;
        let reply = client.request(request).map_err(|error| error.to_string())?;
        done(reply)
    }

    fn keep(&self, request: &Request) -> Result<(), String> {
        let mut link = self.link.lock().map_err(|_| LOST.to_owned())?;
        if let Some(client) = link.as_mut() {
            match client.request(request) {
                Ok(reply) => return done(reply),
                Err(_) => *link = None,
            }
        }
        let mut client = Client::connect(&self.path).map_err(|error| error.to_string())?;
        let reply = client.request(request).map_err(|error| error.to_string())?;
        *link = Some(client);
        done(reply)
    }
}

impl Compositor for Sparky {
    fn lock(&self) -> Result<(), String> {
        self.session(SessionAction::Lock)
    }

    fn unlock(&self) -> Result<(), String> {
        self.session(SessionAction::Unlock)
    }

    fn idle(&self, hold: Hold) -> Result<(), String> {
        let hold = match hold {
            Hold::Free => IdleHold::Free,
            Hold::Held => IdleHold::Held,
        };
        self.keep(&Request::Idle { hold })
    }
}

pub fn watch(path: PathBuf, sender: Sender<Event>) {
    thread::spawn(move || {
        if let Ok(notices) = Client::connect(&path).and_then(Client::subscribe) {
            for input in notices.filter_map(input) {
                if sender.blocking_send(Event::Seen(input)).is_err() {
                    return;
                }
            }
        }
        let _ = sender.blocking_send(Event::CompositorGone);
    });
}

fn done(reply: Reply) -> Result<(), String> {
    match reply {
        Reply::Done => Ok(()),
        Reply::Error { message } => Err(message),
        _ => Err(UNEXPECTED.to_owned()),
    }
}

fn input(notice: Notice) -> Option<Input> {
    match notice {
        Notice::Locked => Some(Input::Locked),
        Notice::Unlocked => Some(Input::Unlocked),
        Notice::WorkspacesChanged => None,
    }
}
