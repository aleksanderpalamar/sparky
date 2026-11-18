use std::fs;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;

use sparky_resources::application::Fault;
use sparky_resources::application::Observer;
use sparky_resources::application::hardware::fingerprint;
use sparky_resources::daemon::brain::Brain;
use sparky_resources::daemon::notify::notify;
use sparky_resources::daemon::{Server, Service, Stop, Windows, follow, sample_forever};
use sparky_resources::persistence::Knowledge;
use sparky_resources::platform::{DiskStorage, LinuxExecutor, LinuxFiles};
use sparky_resources::protocol::socket;
use sparky_resources::safety::{Ledger, Outcome};
use sparky_resources::telemetry::Collector;

fn main() -> ExitCode {
    let Some(path) = socket::from_env() else {
        eprintln!("sparky-resources: XDG_RUNTIME_DIR ausente ou inválido");
        return ExitCode::FAILURE;
    };
    let stop = match Stop::block() {
        Ok(stop) => stop,
        Err(error) => return failed("bloquear sinais", error),
    };
    let server = match Server::bind(&path) {
        Ok(server) => server,
        Err(error) => return failed("abrir o socket", error),
    };
    let mut service = Service::default();
    if !recover(&path) {
        service.failed(Fault::RestoreUnconfirmed);
    }
    service.ready();
    let service = Arc::new(Mutex::new(service));
    let windows = Windows::default();
    if let Some(compositor) = sparky_ipc::socket::from_env() {
        follow(compositor, windows.clone());
    }
    let collector = Collector::new(LinuxFiles);
    let hardware = fingerprint(&LinuxFiles, collector.system().gpu.measured().is_some());
    let knowledge = DiskStorage::from_env()
        .and_then(|directory| DiskStorage::open(directory).ok())
        .map(Knowledge::new);
    sample_forever(
        Observer::new(collector),
        Brain::new(knowledge, hardware),
        windows,
        Arc::clone(&service),
    );
    let shared = Arc::clone(&service);
    let socket_path = server.path().to_owned();
    thread::spawn(move || server.serve(&shared));
    notify("READY=1");
    let outcome = stop.wait();
    let _ = fs::remove_file(&socket_path);
    match outcome {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => failed("esperar sinais", error),
    }
}

fn failed(action: &str, error: impl std::fmt::Display) -> ExitCode {
    eprintln!("sparky-resources: falha ao {action}: {error}");
    ExitCode::FAILURE
}

fn recover(socket: &std::path::Path) -> bool {
    let Some(directory) = socket
        .parent()
        .map(|runtime| runtime.join("sparky-resources"))
    else {
        return true;
    };
    let Ok(storage) = DiskStorage::open(directory) else {
        return true;
    };
    let outcomes = Ledger::open(storage).recover(&LinuxExecutor::new(LinuxFiles));
    !outcomes.contains(&Outcome::Unconfirmed)
}
