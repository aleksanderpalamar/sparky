use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::mpsc::{Receiver, RecvTimeoutError, sync_channel};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::diagnostics::metrics::Metrics;
use crate::diagnostics::resources::Memory;

use super::files::LogFile;
use super::queue::{QUEUE_DEPTH, Queue};

const SAMPLE_INTERVAL: Duration = Duration::from_secs(5);
const FILE_LIMIT: u64 = 2 * 1024 * 1024;

pub struct Guard {
    stop: Arc<AtomicBool>,
    done: Receiver<()>,
    worker: Option<JoinHandle<()>>,
}

pub fn start(directory: Option<&Path>, metrics: Arc<Metrics>) -> io::Result<(Queue, Guard)> {
    let destination = destination(directory);
    let (sender, receiver) = sync_channel(QUEUE_DEPTH);
    let (finished, done) = sync_channel(1);
    let dropped = Arc::new(AtomicU64::new(0));
    let queue = Queue::new(sender, dropped.clone());
    let stop = Arc::new(AtomicBool::new(false));
    let stopping = stop.clone();
    let worker = thread::Builder::new()
        .name("sparky-diagnostics".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            run(destination, receiver, &metrics, &dropped, &stopping);
            let _ = finished.send(());
        })?;
    Ok((
        queue,
        Guard {
            stop,
            done,
            worker: Some(worker),
        },
    ))
}

fn destination(directory: Option<&Path>) -> Box<dyn Write + Send> {
    let file = directory
        .ok_or_else(|| io::Error::other("XDG_STATE_HOME e HOME sem caminho absoluto"))
        .and_then(|path| LogFile::open(path, FILE_LIMIT));
    match file {
        Ok(file) => Box::new(file),
        Err(error) => {
            eprintln!("sparky: logs persistentes indisponíveis, usando stderr: {error}");
            Box::new(io::stderr())
        }
    }
}

fn run(
    mut destination: Box<dyn Write + Send>,
    receiver: Receiver<Vec<u8>>,
    metrics: &Metrics,
    dropped: &AtomicU64,
    stop: &AtomicBool,
) {
    let mut next_sample = Instant::now();
    loop {
        if Instant::now() >= next_sample {
            write(&mut destination, sample(metrics, dropped).as_bytes());
            flush(&mut destination);
            next_sample = Instant::now() + SAMPLE_INTERVAL;
        }
        if stop.load(Relaxed) {
            break;
        }
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(record) => write(&mut destination, &record),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    for record in receiver.try_iter().take(QUEUE_DEPTH) {
        write(&mut destination, &record);
    }
    write(&mut destination, sample(metrics, dropped).as_bytes());
    flush(&mut destination);
}

fn sample(metrics: &Metrics, dropped: &AtomicU64) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!(
        "unix_ms={timestamp} INFO sparky_diagnostics pid={} version={} {} {} dropped_logs={}\n",
        std::process::id(),
        env!("CARGO_PKG_VERSION"),
        metrics.snapshot(),
        Memory::read(),
        dropped.load(Relaxed),
    )
}

fn write(destination: &mut Box<dyn Write + Send>, record: &[u8]) {
    if let Err(error) = destination.write_all(record) {
        eprintln!("sparky: falha ao gravar diagnóstico, usando stderr: {error}");
        *destination = Box::new(io::stderr());
        let _ = destination.write_all(record);
    }
}

fn flush(destination: &mut Box<dyn Write + Send>) {
    if let Err(error) = destination.flush() {
        eprintln!("sparky: falha ao sincronizar diagnóstico, usando stderr: {error}");
        *destination = Box::new(io::stderr());
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.stop.store(true, Relaxed);
        if self.done.recv_timeout(Duration::from_secs(1)).is_err() {
            eprintln!("sparky: diagnóstico não terminou em 1 segundo");
            return;
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
