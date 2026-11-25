use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::brain::Brain;
use super::compositor::Windows;
use super::conduct::Orchestra;
use super::forgetting::Forgetting;
use super::herald::Herald;
use super::notify::notify;
use super::service::Service;
use super::worker::Worker;
use crate::application::budget::{Usage, assess, cpu_percent, usage};
use crate::application::footprint::{Footprint, memory_limit};
use crate::application::{Fault, Link, Observer};
use crate::domain::ServiceState;
use crate::platform::{DesktopNotifier, LinuxFiles, clock_ticks};
use crate::ports::{Files, Storage};
use crate::telemetry::cadence::interval;

const BUDGET_WINDOW: Duration = Duration::from_secs(60);

struct Sampler<F, S, L> {
    started: Instant,
    observer: Observer<F>,
    brain: Brain<S>,
    orchestra: Orchestra<L>,
    windows: Windows,
    service: Arc<Mutex<Service>>,
    spending: Spending,
    herald: Herald<DesktopNotifier>,
    forgetting: Arc<Forgetting>,
}

pub fn sample_forever<F, S, L>(
    observer: Observer<F>,
    brain: Brain<S>,
    orchestra: Orchestra<L>,
    windows: Windows,
    service: Arc<Mutex<Service>>,
    forgetting: Arc<Forgetting>,
) -> Worker<bool>
where
    F: Files + Send + 'static,
    S: Storage + Send + 'static,
    L: Storage + Send + 'static,
{
    lock(&service).learned(brain.validity());
    let sampler = Sampler {
        started: Instant::now(),
        observer,
        brain,
        orchestra,
        windows,
        service,
        spending: Spending::start(),
        herald: Herald::new(DesktopNotifier {
            application: crate::i18n::text("notice-app"),
        }),
        forgetting,
    };
    Worker::spawn(sampler, Sampler::step, Sampler::finish)
}

fn lock(service: &Mutex<Service>) -> MutexGuard<'_, Service> {
    service.lock().unwrap_or_else(PoisonError::into_inner)
}

impl<F: Files, S: Storage, L: Storage> Sampler<F, S, L> {
    fn step(&mut self) -> Duration {
        self.forget_if_asked();
        let current = self.windows.snapshot();
        let link = match current.is_some() {
            true => Link::Available,
            false => Link::Lost,
        };
        let observation = self.observer.observe(Instant::now(), current);
        let insights = self
            .brain
            .insights(self.started.elapsed().as_secs_f64(), &observation);
        let (training, authority) = {
            let mut service = lock(&self.service);
            service.observe(observation.activities, link, insights);
            self.orchestra.protect(service.protected());
            (service.training(), service.state().authority())
        };
        self.orchestra
            .tick(&observation, authority, self.started.elapsed().as_secs());
        self.publish();
        let active = self.brain.train(training, self.started.elapsed().as_secs());
        let pause = settle(lock(&self.service), &mut self.spending, &self.brain, active);
        notify("WATCHDOG=1");
        pause
    }

    fn forget_if_asked(&mut self) {
        if !self.forgetting.take() {
            return;
        }
        let removed = self.brain.forget();
        if let Err(error) = &removed {
            eprintln!("sparky-resources: conhecimento não apagado: {error}");
        }
        self.forgetting.finish(removed.is_ok());
    }

    fn publish(&mut self) {
        let mut service = lock(&self.service);
        let conductor = &self.orchestra.conductor;
        let decisions = conductor.decisions();
        self.herald
            .hear(&decisions, conductor.recorded(), service.notifying());
        service.decided(decisions);
        service.shielded(self.orchestra.guarded(), conductor.adjusting());
        match conductor.unconfirmed() {
            true => service.failed(Fault::RestoreUnconfirmed),
            false => service.restored(),
        }
    }

    fn finish(mut self) -> bool {
        self.orchestra.conductor.shutdown()
    }
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
