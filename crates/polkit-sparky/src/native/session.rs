use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;

use super::{api, signals};
use crate::domain::{Event, Secret};

pub fn run(
    uid: u32,
    cookie: Secret,
    responses: Receiver<Secret>,
    cancelled: Arc<AtomicBool>,
    events: UnboundedSender<Event>,
) {
    let context = glib::MainContext::new();
    let result =
        context.with_thread_default(|| conversation(uid, &cookie, &responses, &cancelled, &events));
    let failure = result
        .map_err(|error| error.to_string())
        .and_then(|result| result);
    if let Err(error) = failure {
        let _ = events.send(Event::Failed(error));
    }
}

fn conversation(
    uid: u32,
    cookie: &Secret,
    responses: &Receiver<Secret>,
    cancelled: &AtomicBool,
    events: &UnboundedSender<Event>,
) -> Result<(), String> {
    let api = api::api()?;
    let session = api.session(uid, cookie)?;
    let done = Rc::new(Cell::new(false));
    signals::connect(&session, events.clone(), done.clone());
    api.initiate(&session);
    let context = glib::MainContext::ref_thread_default();
    while !done.get() {
        while context.pending() {
            context.iteration(false);
        }
        if done.get() {
            break;
        }
        if cancelled.load(Ordering::SeqCst) || events.is_closed() {
            api.cancel(&session);
            break;
        }
        match responses.try_recv() {
            Ok(response) => match api.respond(&session, &response) {
                Ok(()) => {}
                Err(error) => {
                    api.cancel(&session);
                    return Err(error);
                }
            },
            Err(TryRecvError::Disconnected) => {
                api.cancel(&session);
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}
