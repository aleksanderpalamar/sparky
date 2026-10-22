mod api;
mod session;
mod signals;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;

use crate::domain::{Event, Secret};
use crate::ports::{Authenticator, Conversation, Responses};

#[derive(Default)]
pub struct Native {
    workers: Mutex<Vec<Worker>>,
}

struct Worker {
    cancelled: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

struct Control {
    cancelled: Arc<AtomicBool>,
    responses: mpsc::Sender<Secret>,
}

impl Responses for Control {
    fn respond(&self, response: Secret) -> Result<(), String> {
        self.responses
            .send(response)
            .map_err(|_| "A autenticação já terminou".to_owned())
    }
}

impl Drop for Control {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

impl Authenticator for Native {
    fn start(&self, uid: u32, cookie: &Secret) -> Result<Conversation, String> {
        let mut workers = self.workers.lock().map_err(|error| error.to_string())?;
        workers.retain(|worker| !worker.thread.is_finished());
        let (sender, receiver) = mpsc::channel();
        let (events, incoming) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let cancelled = Arc::new(AtomicBool::new(false));
        let stop = cancelled.clone();
        let cookie = cookie.clone();
        let thread = std::thread::Builder::new()
            .name("polkit-conversation".to_owned())
            .spawn(move || session::run(uid, cookie, receiver, stop, events))
            .map_err(|error| error.to_string())?;
        workers.push(Worker {
            cancelled: cancelled.clone(),
            thread,
        });
        Ok(Conversation {
            responses: Box::new(Control {
                cancelled,
                responses: sender,
            }),
            events: incoming,
        })
    }

    fn shutdown(&self) {
        let Ok(mut workers) = self.workers.lock() else {
            return;
        };
        for worker in workers.iter() {
            worker.cancelled.store(true, Ordering::SeqCst);
        }
        for worker in workers.drain(..) {
            let _ = worker.thread.join();
        }
    }
}

pub fn available() -> Result<(), String> {
    api::api().map(|_| ())
}

pub fn username(uid: u32) -> String {
    api::api()
        .map(|api| api.username(uid))
        .unwrap_or_else(|_| format!("UID {uid}"))
}

#[cfg(test)]
mod tests {
    use crate::domain::Secret;
    use glib::prelude::ObjectExt;

    #[test]
    fn a_api_nativa_cria_sessao_sem_iniciar_pam_ou_abrir_janela() {
        let api = super::api::api().unwrap();
        let session = api
            .session(0, &Secret::new("fixture-sem-autenticacao".to_owned()))
            .unwrap();
        assert_eq!(session.type_().name(), "PolkitAgentSession");
        assert!(!api.username(0).is_empty());
    }
}
