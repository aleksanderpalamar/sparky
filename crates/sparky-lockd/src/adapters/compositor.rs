use std::path::PathBuf;
use std::thread;

use sparky_ipc::{Client, Event as Notice, IdleHold, Reply, Request, SessionAction};
use tokio::sync::mpsc::Sender;

use crate::app::Event;
use crate::domain::guard::Input;
use crate::domain::inhibitions::Hold;
use crate::ports::Compositor;

const UNEXPECTED: &str = "resposta inesperada do sparky";

pub struct Sparky {
    path: PathBuf,
}

impl Sparky {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn session(&self, action: SessionAction) -> Result<(), String> {
        self.ask(&Request::Session { action })
    }

    fn ask(&self, request: &Request) -> Result<(), String> {
        let mut client = Client::connect(&self.path).map_err(|error| error.to_string())?;
        match client.request(request) {
            Ok(Reply::Done) => Ok(()),
            Ok(Reply::Error { message }) => Err(message),
            Ok(_) => Err(UNEXPECTED.to_owned()),
            Err(error) => Err(error.to_string()),
        }
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
        self.ask(&Request::Idle { hold })
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

fn input(notice: Notice) -> Option<Input> {
    match notice {
        Notice::Locked => Some(Input::Locked),
        Notice::Unlocked => Some(Input::Unlocked),
        Notice::WorkspacesChanged => None,
    }
}
