use sparky_files::adapters::clipboard::SERVE;
use sparky_files::adapters::clipboard_request::run_from_stdin;

fn main() -> iced::Result {
    if std::env::args().nth(1).as_deref() == Some(SERVE) {
        std::process::exit(run_from_stdin());
    }
    sparky_files::app::run()
}
