use std::process::Command;
use std::time::{Duration, Instant};

const ROUNDS: usize = 30;
const GOAL: Duration = Duration::from_millis(500);

fn main() {
    let mut times: Vec<Duration> = (0..ROUNDS).map(|_| run()).collect();
    times.sort();
    let median = times[ROUNDS / 2];
    let slowest = times[ROUNDS - 1];
    let high = times[ROUNDS * 95 / 100];
    println!("sparky-fetch --plain (sem IPC), {ROUNDS} rodadas");
    println!(
        "  mediana {:.1} ms · p95 {:.1} ms · pior {:.1} ms",
        ms(median),
        ms(high),
        ms(slowest)
    );
    println!("  memória máxima (RSS) {} KiB", peak_kib());
    let verdict = match median <= GOAL {
        true => "dentro da meta",
        false => "acima da meta",
    };
    println!("  meta {} ms: {verdict}", GOAL.as_millis());
}

fn run() -> Duration {
    let start = Instant::now();
    let status = Command::new(env!("CARGO_BIN_EXE_sparky-fetch"))
        .args(["--plain", "--no-config"])
        .env_remove("SPARKY_SOCKET")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .unwrap_or_else(|error| panic!("sparky-fetch não rodou: {error}"));
    let elapsed = start.elapsed();
    assert!(status.status.success(), "sparky-fetch falhou");
    elapsed
}

fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn peak_kib() -> i64 {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    let result = unsafe { libc::getrusage(libc::RUSAGE_CHILDREN, usage.as_mut_ptr()) };
    match result {
        0 => unsafe { usage.assume_init() }.ru_maxrss,
        _ => -1,
    }
}
