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
