use sparky_ui::notice::Notice;

use super::page::Page;
use super::state::{Confirm, Focus, Monitor, Target};
use crate::ports::{Claim, Identity, SignalError, Stop};

impl Monitor {
    pub fn ask(&mut self, stop: Stop) {
        let Some(target) = self.target() else {
            self.notice = Some(Notice::Error(
                "Escolha um processo com j/k antes de encerrar".to_owned(),
            ));
            return;
        };
        match self.claim(&target) {
            Ok(claims) => {
                self.confirm = Some(Confirm {
                    target,
                    stop,
                    claims,
                })
            }
            Err(error) => self.notice = Some(Notice::Error(error.to_string())),
        }
    }

    pub fn confirm(&mut self) {
        let Some(confirm) = self.confirm.take() else {
            return;
        };
        self.notice = Some(match send_all(&confirm.claims, confirm.stop) {
            Ok(()) => Notice::Info(done(&confirm.target, confirm.stop)),
            Err(error) => Notice::Error(error.to_string()),
        });
    }

    pub fn escape(&mut self) {
        if self.confirm.take().is_some() {
            return;
        }
        if self.page != Page::Dashboard {
            self.open(Page::Dashboard);
        }
    }

    fn claim(&self, target: &Target) -> Result<Vec<Box<dyn Claim>>, SignalError> {
        let signals = self.services.signals.as_ref();
        match target {
            Target::Process { pid, .. } => {
                let identity = self.identity(*pid).ok_or(SignalError::Gone)?;
                Ok(vec![signals.claim(identity)?])
            }
            Target::App { pids, .. } => {
                let claims: Vec<_> = pids
                    .iter()
                    .filter_map(|pid| self.identity(*pid))
                    .filter_map(|identity| signals.claim(identity).ok())
                    .collect();
                match claims.is_empty() {
                    true => Err(SignalError::Gone),
                    false => Ok(claims),
                }
            }
        }
    }

    fn identity(&self, pid: u32) -> Option<Identity> {
        self.processes
            .iter()
            .find(|row| row.pid == pid)
            .map(|row| Identity {
                pid,
                started: row.started,
            })
    }

    fn target(&self) -> Option<Target> {
        match self.active_table()? {
            Focus::Processes => {
                let pid = self.selected_process?;
                let row = self.processes.iter().find(|row| row.pid == pid)?;
                Some(Target::Process {
                    pid,
                    name: row.name.clone(),
                })
            }
            Focus::Applications => {
                let id = self.selected_app.as_ref()?;
                let app = self.apps.iter().find(|app| &app.id == id)?;
                Some(Target::App {
                    name: app.name.clone(),
                    pids: app.pids.clone(),
                })
            }
        }
    }
}

pub fn question(confirm: &Confirm) -> String {
    let verb = match confirm.stop {
        Stop::Terminate => "Encerrar",
        Stop::Kill => "Forçar o fim de",
    };
    format!(
        "{verb} {}? Enter confirma · Esc cancela",
        describe(&confirm.target)
    )
}

fn done(target: &Target, stop: Stop) -> String {
    match stop {
        Stop::Terminate => format!("Pedido de encerramento enviado a {}", describe(target)),
        Stop::Kill => format!("{} foi forçado a fechar", describe(target)),
    }
}

fn describe(target: &Target) -> String {
    match target {
        Target::Process { pid, name } => format!("{name} (PID {pid})"),
        Target::App { name, pids } if pids.len() == 1 => format!("{name} (1 processo)"),
        Target::App { name, pids } => format!("{name} ({} processos)", pids.len()),
    }
}

fn send_all(claims: &[Box<dyn Claim>], stop: Stop) -> Result<(), SignalError> {
    let results: Vec<_> = claims.iter().map(|claim| claim.send(stop)).collect();
    if results.iter().any(Result::is_ok) {
        return Ok(());
    }
    results
        .into_iter()
        .find_map(Result::err)
        .map_or(Ok(()), Err)
}
