use std::time::Duration;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tokio::io::BufReader;
use tokio::net::unix::OwnedReadHalf;
use tokio::process::Child;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::domain::job::Report;
use crate::domain::machine::Event;
use crate::service::link::Input;
use crate::service::workers::{Link, next};

const GRACE: Duration = Duration::from_secs(10);

pub async fn greeter(link: Link, mut stop: oneshot::Receiver<()>, inputs: UnboundedSender<Input>) {
    let Link {
        mut child,
        reader,
        writer,
    } = link;
    tokio::spawn(log_reports(reader));
    tokio::select! {
        _ = child.wait() => {}
        Ok(()) = &mut stop => {
            if tokio::time::timeout(GRACE, child.wait()).await.is_err() {
                eprintln!("sparky-login: a tela de login não fechou sozinha; encerrando");
                terminate(&child);
                let _ = child.wait().await;
            }
        }
    }
    drop(writer);
    let _ = inputs.send(Input::Event(Event::GreeterGone));
}

pub async fn login(
    mut child: Child,
    mut reader: BufReader<OwnedReadHalf>,
    inputs: UnboundedSender<Input>,
    attempt: u64,
) {
    while let Some(report) = next::<Report>(&mut reader).await {
        let event = match report {
            Report::Authenticated => Event::Authenticated,
            Report::Refused { refusal } => Event::Refused(refusal),
            other => {
                describe(&other);
                continue;
            }
        };
        if inputs.send(Input::Login { attempt, event }).is_err() {
            return;
        }
    }
    let _ = child.wait().await;
    let _ = inputs.send(Input::Login {
        attempt,
        event: Event::SessionEnded,
    });
}

async fn log_reports(mut reader: BufReader<OwnedReadHalf>) {
    while let Some(report) = next::<Report>(&mut reader).await {
        describe(&report);
    }
}

fn describe(report: &Report) {
    match report {
        Report::Running => eprintln!("sparky-login: sessão aberta"),
        Report::Ended => eprintln!("sparky-login: sessão encerrada"),
        Report::Failed { message } => eprintln!("sparky-login: a sessão falhou: {message}"),
        Report::Authenticated | Report::Refused { .. } => {}
    }
}

fn terminate(child: &Child) {
    if let Some(pid) = child.id().and_then(|pid| i32::try_from(pid).ok()) {
        let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
    }
}
