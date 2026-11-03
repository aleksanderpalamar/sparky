use std::process::ExitCode;

use sparky_ipc::socket;
use sparky_lockd::adapters::compositor::{Sparky, watch};
use sparky_lockd::adapters::logind::SystemLogind;
use sparky_lockd::adapters::screensaver;
use sparky_lockd::app;
use tokio::sync::mpsc;

const CAPACITY: usize = 16;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let Some(path) = socket::from_env() else {
        eprintln!("sparky-lockd: fora de uma sessão do sparky");
        return ExitCode::FAILURE;
    };
    let logind = match connect().await {
        Ok(logind) => logind,
        Err(error) => {
            eprintln!("sparky-lockd: logind indisponível: {error}");
            return ExitCode::FAILURE;
        }
    };
    let (sender, receiver) = mpsc::channel(CAPACITY);
    if let Err(error) = logind.watch(sender.clone()).await {
        eprintln!("sparky-lockd: sinais do logind indisponíveis: {error}");
        return ExitCode::FAILURE;
    }
    let _screensaver = offer_screensaver(sender.clone()).await;
    watch(path.clone(), sender);
    app::run(&logind, &Sparky::new(path), receiver).await;
    ExitCode::SUCCESS
}

async fn offer_screensaver(sender: mpsc::Sender<app::Event>) -> Option<zbus::Connection> {
    let offered = async {
        let connection = zbus::Connection::session().await?;
        screensaver::serve(&connection, sender).await?;
        Ok::<_, zbus::Error>(connection)
    };
    match offered.await {
        Ok(connection) => Some(connection),
        Err(error) => {
            eprintln!("sparky-lockd: sem {}: {error}", screensaver::NAME);
            None
        }
    }
}

async fn connect() -> zbus::Result<SystemLogind> {
    let connection = zbus::Connection::system().await?;
    SystemLogind::connect(&connection).await
}
