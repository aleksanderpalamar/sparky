use std::io::{ErrorKind, Read};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crate::linux::host::Ran;

const POLL: Duration = Duration::from_millis(5);

pub fn run(program: &str, arguments: &[&str], timeout: Duration) -> Ran {
    let deadline = Instant::now() + timeout;
    let spawned = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ran::Missing,
        Err(error) => return Ran::Failed(error.to_string()),
    };
    let Some(stdout) = child.stdout.take() else {
        return stop(&mut child, Ran::Failed(ErrorKind::BrokenPipe.to_string()));
    };
    let output = drain(stdout);
    match wait(&mut child, deadline) {
        Ok(status) if status.success() => collected(&output, deadline),
        Ok(status) => Ran::Failed(status.to_string()),
        Err(ran) => ran,
    }
}

fn drain(mut stdout: ChildStdout) -> Receiver<Result<String, String>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut text = String::new();
        let result = stdout
            .read_to_string(&mut text)
            .map(|_| text)
            .map_err(|error| error.to_string());
        let _ = sender.send(result);
    });
    receiver
}

fn wait(child: &mut Child, deadline: Instant) -> Result<ExitStatus, Ran> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() >= deadline => return Err(stop(child, Ran::TimedOut)),
            Ok(None) => thread::sleep(POLL),
            Err(error) => return Err(stop(child, Ran::Failed(error.to_string()))),
        }
    }
}

fn collected(output: &Receiver<Result<String, String>>, deadline: Instant) -> Ran {
    let left = deadline.saturating_duration_since(Instant::now());
    match output.recv_timeout(left) {
        Ok(Ok(text)) => Ran::Output(text),
        Ok(Err(error)) => Ran::Failed(error),
        Err(_) => Ran::TimedOut,
    }
}

fn stop(child: &mut Child, ran: Ran) -> Ran {
    let killed = child.kill();
    let reaped = child.wait();
    match (killed, reaped) {
        (Ok(()), Ok(_)) => ran,
        (Err(error), _) | (_, Err(error)) => Ran::Failed(error.to_string()),
    }
}
