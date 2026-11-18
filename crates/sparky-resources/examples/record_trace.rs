use std::fs::File;
use std::io::{BufWriter, Write};
use std::thread;
use std::time::{Duration, Instant};

use sparky_resources::application::features;
use sparky_resources::platform::LinuxFiles;
use sparky_resources::predict::trace::{HEADER, line};
use sparky_resources::telemetry::Collector;

fn main() -> std::io::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let (Some(path), Some(seconds), Some(step)) =
        (arguments.next(), arguments.next(), arguments.next())
    else {
        eprintln!("uso: record_trace <arquivo.csv> <duracao_s> <intervalo_ms>");
        std::process::exit(2);
    };
    let duration = Duration::from_secs(seconds.parse().unwrap_or(60));
    let step = Duration::from_millis(step.parse().unwrap_or(2000));
    let collector = Collector::new(LinuxFiles);
    let mut output = BufWriter::new(File::create(path)?);
    writeln!(output, "{HEADER}")?;
    let started = Instant::now();
    let mut previous = collector.system().cpu.measured().copied();
    while started.elapsed() < duration {
        thread::sleep(step);
        let sample = collector.system();
        writeln!(
            output,
            "{}",
            line(
                started.elapsed().as_secs_f64(),
                &features(previous, &sample)
            )
        )?;
        previous = sample.cpu.measured().copied().or(previous);
    }
    output.flush()
}
