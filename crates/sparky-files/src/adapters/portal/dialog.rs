use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::domain::choice::Request;
use crate::domain::choice::wire::{decode_outcome, encode_request};
use crate::ports::chooser::{Closer, Dialogs, Shown};

pub const CHOOSE: &str = "--choose";

pub struct ProcessDialogs {
    program: PathBuf,
}

impl ProcessDialogs {
    pub fn new(program: PathBuf) -> Self {
        Self { program }
    }
}

impl Dialogs for ProcessDialogs {
    fn show(&self, request: &Request, closer: &Closer) -> Shown {
        let spawned = Command::new(&self.program)
            .arg(CHOOSE)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn();
        let Ok(mut child) = spawned else {
            return Shown::Failed;
        };
        let sent = child
            .stdin
            .take()
            .is_some_and(|mut stdin| stdin.write_all(encode_request(request).as_bytes()).is_ok());
        let output = child.stdout.take();
        let child = Arc::new(Mutex::new(child));
        let killer = Arc::clone(&child);
        closer.on_close(move || stop(&killer));
        if !sent {
            stop(&child);
        }
        let mut text = String::new();
        let read = output.is_some_and(|mut output| output.read_to_string(&mut text).is_ok());
        let success = child
            .lock()
            .ok()
            .and_then(|mut child| child.wait().ok())
            .is_some_and(|status| status.success());
        shown(read && sent, success, &text)
    }
}

fn shown(complete: bool, success: bool, text: &str) -> Shown {
    match (complete, success, decode_outcome(text)) {
        (true, true, Some(outcome)) => Shown::Chosen(outcome),
        (_, _, None) if text.trim().is_empty() => Shown::Cancelled,
        _ => Shown::Failed,
    }
}

fn stop(child: &Mutex<Child>) {
    if let Ok(mut child) = child.lock() {
        let _ = child.kill();
    }
}
