use std::cell::RefCell;
use std::io::{BufReader, Cursor};
use std::rc::Rc;

use sparky_login_core::codec::{receive, send};
use sparky_login_core::protocol::Refusal;
use sparky_login_core::sessions::Session;
use zeroize::Zeroizing;

use crate::domain::job::{Job, Order, Report};
use crate::worker::Worker;
use crate::worker::fakes::{FakeAccounts, FakeLauncher, FakePam, Log};

pub struct Cena {
    pub log: Log,
    pub pam: FakePam,
    pub launcher: FakeLauncher,
}

pub fn cena(verdict: Result<(), Refusal>) -> Cena {
    let log: Log = Rc::new(RefCell::new(Vec::new()));
    let pam = FakePam {
        log: Rc::clone(&log),
        verdict,
    };
    let launcher = FakeLauncher {
        log: Rc::clone(&log),
        launched: RefCell::new(Vec::new()),
    };
    Cena { log, pam, launcher }
}

pub fn entrada(job: &Job, orders: &[Order]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let _ = send(&mut bytes, job);
    for order in orders {
        let _ = send(&mut bytes, order);
    }
    bytes
}

pub fn rodar(cena: &Cena, bytes: Vec<u8>) -> (Result<(), String>, Vec<Report>) {
    let worker = Worker {
        pam: &cena.pam,
        accounts: &FakeAccounts,
        launcher: &cena.launcher,
    };
    let mut output = Vec::new();
    let result = worker.run(&mut BufReader::new(Cursor::new(bytes)), &mut output);
    let mut reader = BufReader::new(Cursor::new(output));
    let mut reports = Vec::new();
    while let Ok(Some(report)) = receive::<Report>(&mut reader) {
        reports.push(report);
    }
    (result, reports)
}

pub fn login(username: &str) -> Job {
    let session = Session {
        id: "hyprland".into(),
        name: "Hyprland".into(),
        exec: "Hyprland".into(),
        desktop_names: vec!["Hyprland".into()],
    };
    Job::Login {
        username: username.into(),
        password: Zeroizing::new("segredo".into()),
        session,
        vt: 1,
    }
}

pub fn par(name: &str, value: &str) -> (String, String) {
    (name.to_owned(), value.to_owned())
}
