use std::io::Write;
use std::sync::Arc;

use tracing_subscriber::fmt::MakeWriter;

use crate::diagnostics::metrics::{Metrics, Phase};

#[test]
fn encerramento_drena_a_fila_e_sincroniza_o_arquivo() {
    let directory = tempfile::tempdir().unwrap();
    let (queue, guard) =
        super::worker::start(Some(directory.path()), Arc::new(Metrics::default())).unwrap();
    queue.make_writer().write_all(b"ultimo registro\n").unwrap();
    drop(guard);
    let log = std::fs::read_to_string(directory.path().join("compositor.log")).unwrap();
    assert!(log.contains("ultimo registro\n"));
    assert!(log.contains("sparky_diagnostics"));
    assert!(log.contains("rss_kib="));
}

#[test]
fn coleta_independente_registra_a_fase_sem_executar_loop_wayland() {
    let directory = tempfile::tempdir().unwrap();
    let metrics = Arc::new(Metrics::default());
    metrics.phase(Phase::Render);
    metrics.topology(5, 1, 1, 0);
    let (_queue, guard) = super::worker::start(Some(directory.path()), metrics).unwrap();
    drop(guard);
    let log = std::fs::read_to_string(directory.path().join("compositor.log")).unwrap();
    assert!(log.contains("phase=Render"));
    assert!(log.contains("windows=5 outputs=1 pending=1 timers=0"));
}

#[test]
fn eventos_tracing_sao_persistidos_sem_depender_do_stdout() {
    let directory = tempfile::tempdir().unwrap();
    let (queue, guard) =
        super::worker::start(Some(directory.path()), Arc::new(Metrics::default())).unwrap();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(queue)
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        tracing::warn!(output = "HDMI-A-1", "erro de apresentação de teste");
    });
    drop(guard);
    let log = std::fs::read_to_string(directory.path().join("compositor.log")).unwrap();
    assert!(log.contains("erro de apresentação de teste"));
    assert!(log.contains("HDMI-A-1"));
    assert!(!log.contains('\u{1b}'));
}
