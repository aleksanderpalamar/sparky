use std::fs;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;

use sparky_resources::application::Fault;
use sparky_resources::application::Observer;
use sparky_resources::application::hardware::fingerprint;
use sparky_resources::daemon::brain::Brain;
use sparky_resources::daemon::conduct::Orchestra;
use sparky_resources::daemon::notify::notify;
use sparky_resources::daemon::{Server, Service, Stop, Windows, follow, sample_forever};
use sparky_resources::orchestration::Conductor;
use sparky_resources::persistence::Knowledge;
use sparky_resources::platform::{DiskStorage, LinuxExecutor, LinuxFiles};
use sparky_resources::policy::Capabilities;
use sparky_resources::protocol::socket;
use sparky_resources::safety::{Ledger, Outcome};
use sparky_resources::telemetry::{Collector, read_topology};

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
    let ledger = match runtime(&path) {
        Ok(storage) => Ledger::open(storage),
        Err(error) => return failed("abrir o registro de ajustes", error),
    };
    let mut service = Service::default();
    if !recover(&path) {
        service.failed(Fault::RestoreUnconfirmed);
    }
    service.configure(&settings());
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
        orchestra(ledger),
        windows,
        Arc::clone(&service),
    );
    let shared = Arc::clone(&service);
    let socket_path = server.path().to_owned();
    thread::spawn(move || server.serve(&shared));
    notify("READY=1");
    let outcome = stop.wait();
    recover(&socket_path);
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
    let Ok(storage) = runtime(socket) else {
        return true;
    };
    let outcomes = Ledger::open(storage).recover(&LinuxExecutor::new(LinuxFiles));
    !outcomes.contains(&Outcome::Unconfirmed)
}

fn runtime(socket: &std::path::Path) -> std::io::Result<DiskStorage> {
    let parent = socket.parent().ok_or(std::io::ErrorKind::NotFound)?;
    DiskStorage::open(parent.join("sparky-resources"))
}

fn orchestra(ledger: Ledger<DiskStorage>) -> Orchestra<DiskStorage> {
    let capabilities = Capabilities {
        restore_nice: false,
        io_priority: true,
        affinity: true,
    };
    let spare = read_topology(&LinuxFiles)
        .map(|topology| topology.last_core())
        .unwrap_or_default();
    Orchestra::new(Conductor::new(
        LinuxExecutor::new(LinuxFiles),
        ledger,
        capabilities,
        spare,
    ))
}

fn settings() -> sparky_config::Resources {
    let Ok(home) = std::env::var("HOME") else {
        return sparky_config::Resources::default();
    };
    let path = sparky_config::path(std::env::var("XDG_CONFIG_HOME").ok().as_deref(), &home);
    let Ok(text) = fs::read_to_string(path) else {
        return sparky_config::Resources::default();
    };
    match sparky_config::parse(&text) {
        Ok(config) => config.resources,
        Err(error) => {
            eprintln!("sparky-resources: {error}; usando o padrão seguro");
            sparky_config::Resources::default()
        }
    }
}
