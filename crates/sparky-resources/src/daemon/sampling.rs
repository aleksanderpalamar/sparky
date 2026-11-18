use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use super::brain::Brain;
use super::compositor::Windows;
use super::conduct::Orchestra;
use super::notify::notify;
use super::service::Service;
use crate::application::budget::{Usage, assess, usage};
use crate::application::{Fault, Link, Observer};
use crate::domain::ServiceState;
use crate::platform::{LinuxFiles, clock_ticks};
use crate::ports::{Files, Storage};
use crate::telemetry::cadence::interval;

const BUDGET_WINDOW: Duration = Duration::from_secs(60);

pub fn sample_forever<F, S, L>(
    mut observer: Observer<F>,
    mut brain: Brain<S>,
    mut orchestra: Orchestra<L>,
    windows: Windows,
    service: Arc<Mutex<Service>>,
) where
    F: Files + Send + 'static,
    S: Storage + Send + 'static,
    L: Storage + Send + 'static,
{
    thread::spawn(move || {
        let started = Instant::now();
        let lock = || service.lock().unwrap_or_else(PoisonError::into_inner);
        lock().learned(brain.validity());
        let mut spending = Spending::start();
        loop {
            let current = windows.snapshot();
            let link = if current.is_some() {
                Link::Available
            } else {
                Link::Lost
            };
            let observation = observer.observe(Instant::now(), current);
            let insights = brain.insights(started.elapsed().as_secs_f64(), &observation);
            let (training, forget, authority) = {
                let mut service = lock();
                service.observe(observation.activities, link, insights);
                orchestra.protect(service.protected());
                (
                    service.training(),
                    service.take_forget(),
                    service.state().authority(),
                )
            };
            orchestra.tick(&observation, authority, started.elapsed().as_secs());
            {
                let mut service = lock();
                service.decided(orchestra.conductor.decisions());
                if orchestra.conductor.unconfirmed() {
                    service.failed(Fault::RestoreUnconfirmed);
                }
            }
            if forget {
                brain.forget();
            }
            let active = brain.train(training);
            let pause = settle(lock(), &mut spending, &brain, active);
            notify("WATCHDOG=1");
            thread::sleep(pause);
        }
    });
}

fn settle<S: Storage>(
    mut service: MutexGuard<Service>,
    spending: &mut Spending,
    brain: &Brain<S>,
    active: bool,
) -> Duration {
    service.set_training_active(active);
    service.learned(brain.validity());
    if let Some(budget) = spending.check(service.activities().is_sensitive()) {
        service.spending(budget);
    }
    let pause = interval(service.activities());
    match service.state() {
        ServiceState::Degraded => pause * 2,
        _ => pause,
    }
}

struct Spending {
    since: Instant,
    usage: Option<Usage>,
    ticks: f64,
}

impl Spending {
    fn start() -> Self {
        let ticks = clock_ticks();
        Self {
            since: Instant::now(),
            usage: usage(&LinuxFiles, ticks),
            ticks,
        }
    }

    fn check(&mut self, sensitive: bool) -> Option<crate::application::Budget> {
        let elapsed = self.since.elapsed();
        if elapsed < BUDGET_WINDOW {
            return None;
        }
        let now = usage(&LinuxFiles, self.ticks);
        let budget = self
            .usage
            .zip(now)
            .map(|(before, after)| assess(before, after, elapsed.as_secs_f64(), sensitive));
        self.since = Instant::now();
        self.usage = now;
        budget
    }
}
