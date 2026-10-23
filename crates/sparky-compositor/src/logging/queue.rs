use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::mpsc::SyncSender;

use tracing_subscriber::fmt::MakeWriter;

pub const MAX_RECORD: usize = 8192;
pub const QUEUE_DEPTH: usize = 128;
const TRUNCATED: &[u8] = b" [truncated]\n";

#[derive(Clone)]
pub struct Queue {
    sender: SyncSender<Vec<u8>>,
    dropped: Arc<AtomicU64>,
}

impl Queue {
    pub fn new(sender: SyncSender<Vec<u8>>, dropped: Arc<AtomicU64>) -> Self {
        Self { sender, dropped }
    }
}

pub struct Record {
    queue: Queue,
    bytes: Vec<u8>,
    truncated: bool,
}

impl<'a> MakeWriter<'a> for Queue {
    type Writer = Record;

    fn make_writer(&'a self) -> Self::Writer {
        Record {
            queue: self.clone(),
            bytes: Vec::new(),
            truncated: false,
        }
    }
}

impl Write for Record {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let available = MAX_RECORD.saturating_sub(self.bytes.len());
        self.bytes
            .extend_from_slice(&bytes[..available.min(bytes.len())]);
        self.truncated |= bytes.len() > available;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        if self.bytes.is_empty() {
            return;
        }
        if self.truncated {
            self.bytes.truncate(MAX_RECORD - TRUNCATED.len());
            let valid =
                std::str::from_utf8(&self.bytes).map_or_else(|error| error.valid_up_to(), str::len);
            self.bytes.truncate(valid);
            self.bytes.extend_from_slice(TRUNCATED);
        }
        if self
            .queue
            .sender
            .try_send(std::mem::take(&mut self.bytes))
            .is_err()
        {
            self.queue.dropped.fetch_add(1, Relaxed);
        }
    }
}
