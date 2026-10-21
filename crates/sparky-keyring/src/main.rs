use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match sparky_keyring::daemon::run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-keyring: {error}");
            ExitCode::FAILURE
        }
    }
}
