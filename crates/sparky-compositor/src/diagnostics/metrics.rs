use std::sync::atomic::{AtomicU8, AtomicU64, Ordering::Relaxed};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Starting,
    Dispatch,
    Render,
    Queue,
}

pub struct Metrics {
    start: Instant,
    heartbeat: AtomicU64,
    phase: AtomicU8,
    render_calls: AtomicU64,
    queued: AtomicU64,
    presented: AtomicU64,
    idle: AtomicU64,
    skipped: AtomicU64,
    errors: AtomicU64,
    max_render_us: AtomicU64,
    max_queue_us: AtomicU64,
    last_queue: AtomicU64,
    last_present: AtomicU64,
    windows: AtomicU64,
    outputs: AtomicU64,
    pending: AtomicU64,
    timers: AtomicU64,
}

#[derive(Debug)]
pub struct Snapshot {
    pub uptime_ms: u64,
    pub loop_age_ms: u64,
    pub phase: Phase,
    pub rendered: u64,
    pub queued: u64,
    pub presented: u64,
    pub idle: u64,
    pub skipped: u64,
    pub errors: u64,
    pub max_render_us: u64,
    pub max_queue_us: u64,
    pub last_queue_age_ms: Option<u64>,
    pub last_present_age_ms: Option<u64>,
    pub windows: u64,
    pub outputs: u64,
    pub pending: u64,
    pub timers: u64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            heartbeat: AtomicU64::new(0),
            phase: AtomicU8::new(0),
            render_calls: AtomicU64::new(0),
            queued: AtomicU64::new(0),
            presented: AtomicU64::new(0),
            idle: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            max_render_us: AtomicU64::new(0),
            max_queue_us: AtomicU64::new(0),
            last_queue: AtomicU64::new(0),
            last_present: AtomicU64::new(0),
            windows: AtomicU64::new(0),
            outputs: AtomicU64::new(0),
            pending: AtomicU64::new(0),
            timers: AtomicU64::new(0),
        }
    }
}

impl Metrics {
    pub fn phase(&self, phase: Phase) {
        self.phase.store(phase as u8, Relaxed);
    }

    pub fn heartbeat(&self) {
        self.heartbeat.store(self.elapsed_ms(), Relaxed);
    }

    pub fn topology(&self, windows: usize, outputs: usize, pending: usize, timers: usize) {
        self.windows.store(windows as u64, Relaxed);
        self.outputs.store(outputs as u64, Relaxed);
        self.pending.store(pending as u64, Relaxed);
        self.timers.store(timers as u64, Relaxed);
    }

    pub fn rendered(&self, elapsed: Duration) {
        self.render_calls.fetch_add(1, Relaxed);
        self.max_render_us.fetch_max(micros(elapsed), Relaxed);
    }

    pub fn queued(&self, elapsed: Duration) {
        self.last_queue.store(self.elapsed_ms(), Relaxed);
        self.queued.fetch_add(1, Relaxed);
        self.max_queue_us.fetch_max(micros(elapsed), Relaxed);
    }

    pub fn presented(&self) {
        self.last_present.store(self.elapsed_ms(), Relaxed);
        self.presented.fetch_add(1, Relaxed);
    }

    pub fn idle(&self) {
        self.idle.fetch_add(1, Relaxed);
    }

    pub fn skipped(&self) {
        self.skipped.fetch_add(1, Relaxed);
    }

    pub fn error(&self) {
        self.errors.fetch_add(1, Relaxed);
    }

    pub fn snapshot(&self) -> Snapshot {
        let phase = match self.phase.load(Relaxed) {
            1 => Phase::Dispatch,
            2 => Phase::Render,
            3 => Phase::Queue,
            _ => Phase::Starting,
        };
        Snapshot {
            uptime_ms: self.elapsed_ms(),
            loop_age_ms: self
                .elapsed_ms()
                .saturating_sub(self.heartbeat.load(Relaxed)),
            phase,
            rendered: self.render_calls.load(Relaxed),
            queued: self.queued.load(Relaxed),
            presented: self.presented.load(Relaxed),
            idle: self.idle.load(Relaxed),
            skipped: self.skipped.load(Relaxed),
            errors: self.errors.load(Relaxed),
            max_render_us: self.max_render_us.load(Relaxed),
            max_queue_us: self.max_queue_us.load(Relaxed),
            last_queue_age_ms: (self.queued.load(Relaxed) > 0).then(|| {
                self.elapsed_ms()
                    .saturating_sub(self.last_queue.load(Relaxed))
            }),
            last_present_age_ms: (self.presented.load(Relaxed) > 0).then(|| {
                self.elapsed_ms()
                    .saturating_sub(self.last_present.load(Relaxed))
            }),
            windows: self.windows.load(Relaxed),
            outputs: self.outputs.load(Relaxed),
            pending: self.pending.load(Relaxed),
            timers: self.timers.load(Relaxed),
        }
    }

    fn elapsed_ms(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

fn micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}

impl std::fmt::Display for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "uptime_ms={} loop_age_ms={} phase={:?} render_calls={} queued={} presented={} idle={} skipped={} errors={} max_render_us={} max_queue_us={} last_queue_age_ms={:?} last_present_age_ms={:?} windows={} outputs={} pending={} timers={}",
            self.uptime_ms,
            self.loop_age_ms,
            self.phase,
            self.rendered,
            self.queued,
            self.presented,
            self.idle,
            self.skipped,
            self.errors,
            self.max_render_us,
            self.max_queue_us,
            self.last_queue_age_ms,
            self.last_present_age_ms,
            self.windows,
            self.outputs,
            self.pending,
            self.timers
        )
    }
}
