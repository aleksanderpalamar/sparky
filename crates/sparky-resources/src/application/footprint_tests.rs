use super::budget::{Usage, cpu_percent};
use super::footprint::{Footprint, memory_limit};
use crate::telemetry::fake_files::FakeFiles;

#[test]
fn cpu_do_servico_e_a_fracao_do_tempo_de_parede() {
    let before = Usage {
        cpu_seconds: 1.0,
        rss_kib: 3000,
    };
    let after = Usage {
        cpu_seconds: 1.06,
        ..before
    };
    assert!((cpu_percent(before, after, 60.0) - 0.1).abs() < 1e-9);
    let reset = Usage {
        cpu_seconds: 0.5,
        ..before
    };
    assert_eq!(cpu_percent(before, reset, 60.0), 0.0);
}

#[test]
fn limite_de_memoria_vem_do_cgroup_do_proprio_servico() {
    let unit =
        "/user.slice/user-1000.slice/user@1000.service/background.slice/sparky-resources.service";
    let files = FakeFiles::default()
        .with("/proc/self/cgroup", &format!("0::{unit}\n"))
        .with(&format!("/sys/fs/cgroup{unit}/memory.max"), "67108864\n");
    assert_eq!(memory_limit(&files), Some(65_536));
    let free = FakeFiles::default()
        .with("/proc/self/cgroup", "0::/user.slice\n")
        .with("/sys/fs/cgroup/user.slice/memory.max", "max\n");
    assert_eq!(memory_limit(&free), None);
    assert_eq!(memory_limit(&FakeFiles::default()), None);
}

#[test]
fn a_pegada_vai_e_volta_pelo_protocolo() {
    let footprint = Footprint {
        cpu_percent: 0.04,
        rss_kib: 3124,
        limit_kib: Some(65_536),
    };
    let text = serde_json::to_string(&footprint).unwrap_or_default();
    assert_eq!(
        serde_json::from_str::<Footprint>(&text).ok(),
        Some(footprint)
    );
}
