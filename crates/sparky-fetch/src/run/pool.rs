use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::domain::field::Field;
use crate::domain::status::Status;
use crate::run::Limits;
use crate::sources::Source;

type Finished = (usize, Vec<(Field, Status)>);
type Queue = Arc<Mutex<VecDeque<(usize, Arc<dyn Source>)>>>;

pub fn run(jobs: &[Arc<dyn Source>], wanted: &[Field], limits: Limits) -> Vec<Finished> {
    if jobs.is_empty() {
        return Vec::new();
    }
    let queue: Queue = Arc::new(Mutex::new(jobs.iter().cloned().enumerate().collect()));
    let (sender, receiver) = mpsc::channel();
    let workers = limits.workers.clamp(1, jobs.len());
    for _ in 0..workers {
        spawn(Arc::clone(&queue), sender.clone(), wanted.to_vec());
    }
    drop(sender);
    gather(&receiver, jobs.len(), Instant::now() + limits.timeout)
}

fn spawn(queue: Queue, sender: Sender<Finished>, wanted: Vec<Field>) {
    let started = thread::Builder::new()
        .name("sparky-fetch".to_owned())
        .spawn(move || work(&queue, &sender, &wanted));
    if let Err(error) = started {
        eprintln!("sparky-fetch: {error}");
    }
}

fn work(queue: &Queue, sender: &Sender<Finished>, wanted: &[Field]) {
    while let Some((index, source)) = next(queue) {
        if sender.send((index, source.collect(wanted))).is_err() {
            return;
        }
    }
}

fn next(queue: &Queue) -> Option<(usize, Arc<dyn Source>)> {
    queue.lock().ok()?.pop_front()
}

fn gather(receiver: &Receiver<Finished>, expected: usize, deadline: Instant) -> Vec<Finished> {
    let mut finished = Vec::with_capacity(expected);
    while finished.len() < expected {
        let left = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(left) {
            Ok(result) => finished.push(result),
            Err(_) => break,
        }
    }
    finished
}
