use std::fs;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;

use sparky_resources::application::Observer;
use sparky_resources::application::hardware::fingerprint;
use sparky_resources::daemon::brain::Brain;
use sparky_resources::daemon::{Server, Service, Stop, Windows, follow, sample_forever};
use sparky_resources::persistence::Knowledge;
use sparky_resources::platform::{DiskStorage, LinuxFiles};
use sparky_resources::protocol::socket;
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
