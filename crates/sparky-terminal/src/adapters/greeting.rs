use std::env;
use std::fs;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::Term;
use alacritty_terminal::vte::ansi::Processor;
use sparky_config::Greeting;
use sparky_xdg::programs::installed;

use crate::adapters::listener::Listener;
use crate::domain::greeting::{PROGRAM, arguments, screen_text, wanted};
use crate::domain::launch::{ENVIRONMENT, Launch};

const LIMIT: Duration = Duration::from_millis(1500);
const POLL: Duration = Duration::from_millis(10);

pub fn greet(launch: &Launch, term: &FairMutex<Term<Listener>>, columns: usize) {
    if wanted(launch, preference) != Greeting::Fetch || !installed(PROGRAM) {
        return;
    }
    if let Some(output) = run(PROGRAM, &arguments(columns), LIMIT) {
        feed(term, &output);
    }
}

pub fn feed(term: &FairMutex<Term<Listener>>, output: &[u8]) {
    let mut processor: Processor = Processor::new();
    processor.advance(&mut *term.lock(), &screen_text(output));
}

pub fn run(program: &str, arguments: &[String], limit: Duration) -> Option<Vec<u8>> {
    let mut child = Command::new(program)
        .args(arguments)
        .envs(ENVIRONMENT)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let deadline = Instant::now() + limit;
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut output = Vec::new();
        let _ = sender.send(stdout.read_to_end(&mut output).map(|_| output));
    });
    let finished = wait(&mut child, limit)?;
    let left = deadline.saturating_duration_since(Instant::now());
    let output = receiver.recv_timeout(left).ok()?.ok()?;
    (finished && !output.is_empty()).then_some(output)
}

fn wait(child: &mut Child, limit: Duration) -> Option<bool> {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait().ok()? {
            return Some(status.success());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        thread::sleep(POLL);
    }
}

fn preference() -> Greeting {
    let home = env::var("HOME").unwrap_or_default();
    let config_home = env::var("XDG_CONFIG_HOME").ok();
    fs::read_to_string(sparky_config::path(config_home.as_deref(), &home))
        .ok()
        .and_then(|contents| sparky_config::parse(&contents).ok())
        .map_or(Greeting::Quiet, |config| config.terminal.greeting())
}
