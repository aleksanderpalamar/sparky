use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Instant;

use super::compositor::Windows;
use super::service::Service;
use crate::application::{Baseline, Link, Observer};
use crate::ports::Files;
use crate::telemetry::cadence::interval;

pub fn sample_forever<F: Files + Send + 'static>(
    mut observer: Observer<F>,
    windows: Windows,
    service: Arc<Mutex<Service>>,
) {
    thread::spawn(move || {
        let started = Instant::now();
        let mut baseline = Baseline::default();
        loop {
            let current = windows.snapshot();
            let link = match current {
                Some(_) => Link::Available,
                None => Link::Lost,
            };
            let observation = observer.observe(Instant::now(), current);
            let activities = observation.activities;
            let insights = baseline.observe(started.elapsed().as_secs_f64(), &observation.features);
            service
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .observe(activities, link, insights);
            thread::sleep(interval(activities));
        }
    });
}
