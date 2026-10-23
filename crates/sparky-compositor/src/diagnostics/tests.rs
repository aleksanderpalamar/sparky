use std::time::Duration;

use super::metrics::{Metrics, Phase};
use super::resources::Memory;

#[test]
fn memoria_ausente_nao_e_apresentada_como_zero() {
    let memory = Memory::parse(
        "VmRSS: 4096 kB\nVmSwap: 128 kB\nThreads: 3\n",
        "MemAvailable: 6000 kB\nSwapFree: 3000 kB\n",
        "some avg10=2.50 avg60=1.00\nfull avg10=0.10 avg60=0.00\n",
    );
    assert_eq!(memory.rss_kib, Some(4096));
    assert_eq!(memory.swap_kib, Some(128));
    assert_eq!(memory.peak_kib, None);
    assert_eq!(memory.available_kib, Some(6000));
    assert_eq!(memory.swap_free_kib, Some(3000));
    assert_eq!(memory.threads, Some(3));
    assert_eq!(memory.pressure_some_avg10, Some(2.5));
    assert_eq!(memory.pressure_full_avg10, Some(0.1));
    assert_eq!(Memory::parse("VmRSS: errado kB", "", "").rss_kib, None);
    assert_eq!(
        Memory::parse("", "", "some avg10=errado").pressure_some_avg10,
        None
    );
}

#[test]
fn contadores_e_maximos_acumulam_sem_perder_o_pico() {
    let metrics = Metrics::default();
    for _ in 0..100_000 {
        metrics.rendered(Duration::from_millis(2));
        metrics.queued(Duration::from_micros(100));
        metrics.presented();
    }
    metrics.rendered(Duration::from_millis(80));
    metrics.error();
    metrics.idle();
    metrics.skipped();
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.rendered, 100_001);
    assert_eq!(snapshot.queued, 100_000);
    assert_eq!(snapshot.presented, 100_000);
    assert_eq!(snapshot.errors, 1);
    assert_eq!(snapshot.idle, 1);
    assert_eq!(snapshot.skipped, 1);
    assert_eq!(snapshot.max_render_us, 80_000);
    assert_eq!(snapshot.max_queue_us, 100);
    assert!(snapshot.last_queue_age_ms.is_some());
    assert!(snapshot.last_present_age_ms.is_some());
}

#[test]
fn fase_e_progresso_do_loop_podem_ser_observados_fora_do_loop_grafico() {
    let metrics = Metrics::default();
    assert_eq!(metrics.snapshot().last_queue_age_ms, None);
    metrics.phase(Phase::Render);
    metrics.heartbeat();
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.phase, Phase::Render);
    assert!(snapshot.loop_age_ms < 1000);
    metrics.phase(Phase::Dispatch);
    assert_eq!(metrics.snapshot().phase, Phase::Dispatch);
}
