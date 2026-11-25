use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use super::brain::Brain;
use super::compositor::Windows;
use super::conduct::Orchestra;
use super::herald::Herald;
use super::notify::notify;
use super::service::Service;
use crate::application::budget::{Usage, assess, cpu_percent, usage};
use crate::application::footprint::{Footprint, memory_limit};
use crate::application::{Fault, Link, Observer};
use crate::domain::ServiceState;
use crate::platform::{DesktopNotifier, LinuxFiles, clock_ticks};
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
        let mut herald = Herald::new(DesktopNotifier {
            application: crate::i18n::text("notice-app"),
        });
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
                let decisions = orchestra.conductor.decisions();
                let recorded = orchestra.conductor.recorded();
                herald.hear(&decisions, recorded, service.notifying());
                service.decided(decisions);
                service.shielded(orchestra.guarded(), orchestra.conductor.adjusting());
                match orchestra.conductor.unconfirmed() {
                    true => service.failed(Fault::RestoreUnconfirmed),
                    false => service.restored(),
                }
            }
            if forget {
                brain.forget();
            }
            let active = brain.train(training, started.elapsed().as_secs());
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
    service.progressed(brain.progress());
    if let Some((budget, footprint)) = spending.check(service.activities().is_sensitive()) {
        service.spending(budget);
        service.measured(footprint);
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
    limit: Option<u64>,
}

impl Spending {
    fn start() -> Self {
        let ticks = clock_ticks();
        Self {
            since: Instant::now(),
            usage: usage(&LinuxFiles, ticks),
            ticks,
            limit: memory_limit(&LinuxFiles),
        }
    }

    fn check(&mut self, sensitive: bool) -> Option<(crate::application::Budget, Footprint)> {
        let elapsed = self.since.elapsed();
        if elapsed < BUDGET_WINDOW {
            return None;
        }
        let now = usage(&LinuxFiles, self.ticks);
        let seconds = elapsed.as_secs_f64();
        let result = self.usage.zip(now).map(|(before, after)| {
            let footprint = Footprint {
                cpu_percent: cpu_percent(before, after, seconds),
                rss_kib: after.rss_kib,
                limit_kib: self.limit,
            };
            (assess(before, after, seconds, sensitive), footprint)
        });
        self.since = Instant::now();
        self.usage = now;
        result
    }
}
