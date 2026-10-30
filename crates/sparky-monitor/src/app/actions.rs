use sparky_ui::notice::Notice;

use super::page::Page;
use super::state::{Confirm, Focus, Monitor, Target};
use crate::ports::{SignalError, Signals, Stop};

impl Monitor {
    pub fn ask(&mut self, stop: Stop) {
        match self.target() {
            Some(target) => self.confirm = Some(Confirm { target, stop }),
            None => {
                self.notice = Some(Notice::Error(
                    "Escolha um processo com j/k antes de encerrar".to_owned(),
                ))
            }
        }
    }

    pub fn confirm(&mut self) {
        let Some(Confirm { target, stop }) = self.confirm.take() else {
            return;
        };
        let signals = self.services.signals.as_ref();
        let result = match &target {
            Target::Process { pid, .. } => signals.send(*pid, stop),
            Target::App { pids, .. } => send_all(signals, pids, stop),
        };
        self.notice = Some(match result {
            Ok(()) => Notice::Info(done(&target, stop)),
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

fn send_all(signals: &dyn Signals, pids: &[u32], stop: Stop) -> Result<(), SignalError> {
    let results: Vec<_> = pids.iter().map(|pid| signals.send(*pid, stop)).collect();
    if results.iter().any(Result::is_ok) {
        return Ok(());
    }
    results
        .into_iter()
        .find_map(Result::err)
        .map_or(Ok(()), Err)
}
