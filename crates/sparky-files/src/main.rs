use std::io::Read;

use sparky_files::adapters::clipboard::SERVE;
use sparky_files::adapters::clipboard_request::run_from_stdin;
use sparky_files::adapters::file_manager1;
use sparky_files::adapters::portal::{dialog, service};
use sparky_files::domain::choice::wire::decode_request;

fn main() -> iced::Result {
    match std::env::args().nth(1).as_deref() {
        Some(SERVE) => std::process::exit(run_from_stdin()),
        Some(file_manager1::SERVE) => std::process::exit(served(file_manager1::serve())),
        Some(service::SERVE) => std::process::exit(served(service::serve())),
        Some(dialog::CHOOSE) => return choose(),
        _ => {}
    }
    sparky_files::app::run()
}

fn served(result: std::io::Result<()>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("sparky-files: {error}");
            1
        }
    }
}

fn choose() -> iced::Result {
    let mut text = String::new();
    if std::io::stdin().read_to_string(&mut text).is_err() {
        std::process::exit(1);
    }
    let Some(request) = decode_request(&text) else {
        eprintln!("sparky-files: pedido de escolha inválido");
        std::process::exit(1);
    };
    sparky_files::app::chooser::run(request)
}
