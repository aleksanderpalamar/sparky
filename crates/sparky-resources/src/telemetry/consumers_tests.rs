use std::time::{Duration, Instant};

use super::consumers::Consumers;
use super::fake_files::FakeFiles;

fn stat(pid: u32, name: &str, ticks: u64, started: u64) -> String {
    format!("{pid} ({name}) R 1 0 0 0 -1 0 0 0 0 0 {ticks} 0 0 0 20 0 1 0 {started} 0")
}

fn at(ticks: u64) -> FakeFiles {
    FakeFiles::default().with("/proc/50/stat", &stat(50, "compilador", ticks, 7))
}

fn rate(consumers: &mut Consumers, files: &FakeFiles, now: Instant) -> Option<u64> {
    consumers
        .measure(files, &[50], now)
        .first()
        .map(|(_, usage)| usage.cpu)
}

#[test]
fn mede_o_consumo_entre_duas_leituras() {
    let (mut consumers, start) = (Consumers::default(), Instant::now());
    assert_eq!(rate(&mut consumers, &at(100), start), None);
    assert_eq!(
        rate(&mut consumers, &at(160), start + Duration::from_secs(2)),
        Some(60)
    );
}

#[test]
fn pid_reaproveitado_comeca_do_zero() {
    let (mut consumers, start) = (Consumers::default(), Instant::now());
    consumers.measure(&at(500), &[50], start);
    let reused = FakeFiles::default().with("/proc/50/stat", &stat(50, "novo", 900, 8));
    assert_eq!(
        rate(&mut consumers, &reused, start + Duration::from_secs(2)),
        None
    );
}

#[test]
fn primeira_medicao_apos_o_hiato_nao_acumula_a_cpu_sem_pressao() {
    let (mut consumers, start) = (Consumers::default(), Instant::now());
    rate(&mut consumers, &at(100), start);
    consumers.pause();
    let resumed = start + Duration::from_secs(300);
    assert_eq!(rate(&mut consumers, &at(190), resumed), None);
    assert_eq!(
        rate(&mut consumers, &at(190), resumed + Duration::from_secs(2)),
        Some(0)
    );
}

#[test]
fn mesma_taxa_da_o_mesmo_valor_em_cadencias_diferentes() {
    let burn = |elapsed: Duration| at(1000 + (elapsed.as_secs_f64() * 25.0) as u64);
    for cadence in [Duration::from_millis(250), Duration::from_secs(2)] {
        let (mut consumers, start) = (Consumers::default(), Instant::now());
        let mut last = None;
        for step in 0..=(4_000 / cadence.as_millis() as u32) {
            let elapsed = cadence * step;
            last = rate(&mut consumers, &burn(elapsed), start + elapsed).or(last);
        }
        assert_eq!(last, Some(50), "cadência {cadence:?}");
    }
}

fn with_io(ticks: u64, bytes: u64) -> FakeFiles {
    at(ticks).with(
        "/proc/50/io",
        &format!("rchar: 1\nwchar: 1\nread_bytes: {bytes}\nwrite_bytes: {bytes}\ncancelled_write_bytes: 0\n"),
    )
}

#[test]
fn mede_tambem_o_io_de_disco_no_mesmo_intervalo() {
    let (mut consumers, start) = (Consumers::default(), Instant::now());
    consumers.measure(&with_io(100, 1_000), &[50], start);
    let usage = consumers
        .measure(&with_io(160, 2_000), &[50], start + Duration::from_secs(4))
        .first()
        .map(|(_, usage)| *usage);
    assert_eq!(
        usage.map(|usage| (usage.cpu, usage.io)),
        Some((30, Some(1_000)))
    );
}

#[test]
fn sem_contador_de_io_o_io_fica_ausente() {
    let (mut consumers, start) = (Consumers::default(), Instant::now());
    consumers.measure(&at(100), &[50], start);
    let usage = consumers
        .measure(&at(160), &[50], start + Duration::from_secs(2))
        .first()
        .map(|(_, usage)| *usage);
    assert_eq!(usage.map(|usage| usage.io), Some(None));
}
