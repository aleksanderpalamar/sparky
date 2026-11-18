use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Instant;

use super::brain::Brain;
use super::compositor::Windows;
use super::service::Service;
use crate::application::{Link, Observer};
use crate::ports::{Files, Storage};
use crate::telemetry::cadence::interval;

pub fn sample_forever<F, S>(
    mut observer: Observer<F>,
    mut brain: Brain<S>,
    windows: Windows,
    service: Arc<Mutex<Service>>,
) where
    F: Files + Send + 'static,
    S: Storage + Send + 'static,
{
    thread::spawn(move || {
        let started = Instant::now();
        let lock = || service.lock().unwrap_or_else(PoisonError::into_inner);
        lock().learned(brain.validity());
        loop {
            let current = windows.snapshot();
            let link = if current.is_some() {
                Link::Available
            } else {
                Link::Lost
            };
            let observation = observer.observe(Instant::now(), current);
            let insights = brain.insights(started.elapsed().as_secs_f64(), &observation);
            let (training, forget) = {
                let mut service = lock();
                service.observe(observation.activities, link, insights);
                (service.training(), service.take_forget())
            };
            if forget {
                brain.forget();
            }
            let active = brain.train(training);
            let mut service = lock();
            service.set_training_active(active);
            service.learned(brain.validity());
            drop(service);
            thread::sleep(interval(observation.activities));
        }
    });
}
