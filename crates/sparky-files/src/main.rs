use sparky_files::adapters::clipboard::SERVE;
use sparky_files::adapters::clipboard_request::run_from_stdin;
use sparky_files::adapters::file_manager1;

fn main() -> iced::Result {
    match std::env::args().nth(1).as_deref() {
        Some(SERVE) => std::process::exit(run_from_stdin()),
        Some(file_manager1::SERVE) => std::process::exit(serve_file_manager1()),
        _ => {}
    }
    sparky_files::app::run()
}

fn serve_file_manager1() -> i32 {
    match file_manager1::serve() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("sparky-files: {error}");
            1
        }
    }
}
