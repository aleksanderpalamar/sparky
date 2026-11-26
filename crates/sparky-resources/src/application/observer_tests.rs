use std::time::{Duration, Instant};

use super::observer::Observer;
use crate::domain::Activity;
use crate::telemetry::Collector;
use crate::telemetry::fake_files::FakeFiles;

fn machine(busy: u64) -> FakeFiles {
    FakeFiles::default()
        .with(
            "/proc/stat",
            &format!("cpu  {busy} 0 0 {} 0 0 0 0\n", 1000 - busy),
        )
        .with("/proc/meminfo", "MemTotal: 1000 kB\nMemAvailable: 500 kB\n")
        .with(
            "/proc/1/stat",
            "1 (systemd) S 0 0 0 0 -1 0 0 0 0 0 1 1 0 0 20 0 1 0 1 0",
        )
}

fn with_obs(files: FakeFiles) -> FakeFiles {
    files.with(
        "/proc/900/stat",
        "900 (obs) S 1 0 0 0 -1 0 0 0 0 0 1 1 0 0 20 0 1 0 1 0",
    )
}

#[test]
fn a_primeira_leitura_nao_inventa_uso_de_cpu() {
    let mut observer = Observer::new(Collector::new(machine(0)));
    let observation = observer.observe(Instant::now(), None);
    assert!(observation.activities.contains(Activity::Desktop));
    assert!(observation.sample.cpu.measured().is_some());
}

#[test]
fn reconhece_gravacao_pelos_processos_lidos() {
    let mut observer = Observer::new(Collector::new(with_obs(machine(10))));
    let observation = observer.observe(Instant::now(), Some(Vec::new()));
    assert!(observation.activities.contains(Activity::Recording));
}

#[test]
fn o_contexto_se_mantem_entre_leituras() {
    let mut observer = Observer::new(Collector::new(with_obs(machine(10))));
    let start = Instant::now();
    let first = observer.observe(start, None);
    let second = observer.observe(start + Duration::from_secs(1), None);
    assert_eq!(first.activities, second.activities);
}

struct Scripted {
    answer: crate::context::Sensed,
    asked: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl crate::ports::CaptureSensor for Scripted {
    fn sense(&mut self, _now: Instant) -> crate::context::Sensed {
        self.asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.answer
    }
}

fn sensing(
    files: FakeFiles,
    answer: crate::context::Sensed,
) -> (
    Observer<FakeFiles>,
    std::sync::Arc<std::sync::atomic::AtomicUsize>,
) {
    let asked = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let sensor = Scripted {
        answer,
        asked: std::sync::Arc::clone(&asked),
    };
    (
        Observer::new(Collector::new(files)).sensing(Box::new(sensor)),
        asked,
    )
}

#[test]
fn obs_aberto_e_parado_libera_o_treino() {
    let (mut observer, asked) = sensing(with_obs(machine(10)), crate::context::Sensed::Idle);
    let observation = observer.observe(Instant::now(), Some(Vec::new()));
    assert!(!observation.activities.contains(Activity::Recording));
    assert_eq!(
        observation.capture,
        Some((900, crate::context::Sensed::Idle))
    );
    assert_eq!(asked.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn sem_obs_aberto_o_obs_nao_e_consultado() {
    let (mut observer, asked) = sensing(machine(10), crate::context::Sensed::Capturing);
    let observation = observer.observe(Instant::now(), Some(Vec::new()));
    assert_eq!(observation.capture, None);
    assert_eq!(asked.load(std::sync::atomic::Ordering::SeqCst), 0);
}
