use std::fs;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;

use sparky_resources::daemon::{Server, Service, Stop};
use sparky_resources::protocol::socket;

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
