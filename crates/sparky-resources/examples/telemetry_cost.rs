use std::time::{Duration, Instant};

use sparky_resources::context::{Scene, recognize};
use sparky_resources::platform::LinuxFiles;
use sparky_resources::telemetry::Collector;
use sparky_resources::telemetry::processes::{ProcessTable, scan};

const ROUNDS: usize = 200;

fn main() {
    let collector = Collector::new(LinuxFiles);
    report("amostra do sistema", || {
        collector.system();
    });
    report("varredura de processos", || drop(scan(&LinuxFiles)));
    let mut table = ProcessTable::default();
    table.update(&LinuxFiles, Instant::now());
    report("tabela incremental", || {
        table.update(&LinuxFiles, Instant::now());
    });
    let processes = scan(&LinuxFiles);
    let scene = Scene {
        processes,
        ..Scene::default()
    };
    report("reconhecimento de contexto", || {
        recognize(&scene);
    });
}

fn report(label: &str, mut action: impl FnMut()) {
    let mut samples: Vec<Duration> = (0..ROUNDS)
        .map(|_| {
            let start = Instant::now();
            action();
            start.elapsed()
        })
        .collect();
    samples.sort();
    let percentile = |fraction: f64| samples[((samples.len() - 1) as f64 * fraction) as usize];
    println!(
        "{label:28} p50={:>9.1?} p95={:>9.1?}",
        percentile(0.5),
        percentile(0.95)
    );
}
