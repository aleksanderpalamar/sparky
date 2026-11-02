use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use nix::unistd::{Uid, chown};
use sparky_login_core::protocol::Request;
use tokio::io::BufReader;
use tokio::net::UnixListener;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::UnboundedSender;

use crate::domain::machine::Event;
use crate::service::workers::next;

const PRIVATE: u32 = 0o600;
const PAUSE: Duration = Duration::from_millis(200);

pub enum Input {
    Event(Event),
    Connected(OwnedWriteHalf),
    Login { attempt: u64, event: Event },
}

pub fn bind(path: &Path, owner: u32) -> Result<UnixListener, String> {
    let shown = path.display();
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    }
    let _ = fs::remove_file(path);
    let listener = UnixListener::bind(path).map_err(|error| format!("{shown}: {error}"))?;
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE))
        .map_err(|error| format!("{shown}: {error}"))?;
    chown(path, Some(Uid::from_raw(owner)), None).map_err(|error| format!("{shown}: {error}"))?;
    Ok(listener)
}

pub async fn listen(listener: UnixListener, owner: u32, inputs: UnboundedSender<Input>) {
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(error) => {
                eprintln!("sparky-login: conexão da tela: {error}");
                tokio::time::sleep(PAUSE).await;
                continue;
            }
        };
        if !stream
            .peer_cred()
            .is_ok_and(|credentials| credentials.uid() == owner)
        {
            eprintln!("sparky-login: conexão recusada, não veio da tela de login");
            continue;
        }
        let (reader, writer) = stream.into_split();
        if inputs.send(Input::Connected(writer)).is_err() {
            return;
        }
        tokio::spawn(requests(BufReader::new(reader), inputs.clone()));
    }
}

async fn requests(mut reader: BufReader<OwnedReadHalf>, inputs: UnboundedSender<Input>) {
    while let Some(request) = next::<Request>(&mut reader).await {
        if inputs.send(Input::Event(Event::Request(request))).is_err() {
            return;
        }
    }
}
