use std::sync::{Arc, Barrier, Mutex};
use std::time::Duration;

use super::worker::Worker;

#[test]
fn parada_espera_o_ciclo_em_andamento_e_nada_roda_depois_do_fim() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let started = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let (seen, inside, freed) = (Arc::clone(&log), Arc::clone(&started), Arc::clone(&release));
    let mut first = true;
    let worker = Worker::spawn(
        (),
        move |_| {
            if first {
                first = false;
                inside.wait();
                freed.wait();
            }
            seen.lock().map(|mut log| log.push("ciclo")).ok();
            Duration::from_secs(60)
        },
        {
            let seen = Arc::clone(&log);
            move |_| {
                seen.lock().map(|mut log| log.push("fim")).ok();
                true
            }
        },
    );
    started.wait();
    let stopping = std::thread::spawn(move || worker.finish());
    release.wait();
    assert!(matches!(stopping.join(), Ok(Ok(true))));
    let log = log.lock().map(|log| log.clone()).unwrap_or_default();
    assert_eq!(log.last(), Some(&"fim"));
    assert_eq!(log.iter().filter(|entry| **entry == "fim").count(), 1);
}

#[test]
fn parada_nao_espera_a_pausa_inteira() {
    let worker = Worker::spawn((), |_| Duration::from_secs(3600), |_| 7);
    let begun = std::time::Instant::now();
    assert!(matches!(worker.finish(), Ok(7)));
    assert!(begun.elapsed() < Duration::from_secs(5));
}
