fn main() {
    let result = match std::env::args().nth(1).as_deref() {
        Some("--wait-ready") => sparky_notifications::service::readiness::wait(),
        _ => sparky_notifications::app::run().map_err(|error| error.to_string()),
    };
    if let Err(error) = result {
        eprintln!("sparky-notifications: {error}");
        std::process::exit(1);
    }
}
