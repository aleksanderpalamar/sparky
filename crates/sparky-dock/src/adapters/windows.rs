use std::path::Path;
use std::thread;
use std::time::Duration;

use iced::futures::channel::mpsc::{UnboundedSender, unbounded};
use iced::futures::{SinkExt, Stream, StreamExt};
use sparky_ipc::{Client, IpcError, Reply, Request, socket};

use crate::domain::presence::Seen;

const RECONNECT: Duration = Duration::from_secs(2);
const TIMEOUT: Duration = Duration::from_secs(2);

pub fn changes() -> impl Stream<Item = Vec<Seen>> {
    iced::stream::channel(4, async |mut output| {
        let (sender, mut lists) = unbounded();
        thread::spawn(move || follow(&sender));
        while let Some(list) = lists.next().await {
            if output.send(list).await.is_err() {
                return;
            }
        }
    })
}

fn follow(sender: &UnboundedSender<Vec<Seen>>) {
    let mut reported = None;
    while !sender.is_closed() {
        let ended = match socket::from_env() {
            Some(path) => watch(&path, sender),
            None => Err(IpcError::Unavailable),
        };
        let problem = ended.err().map(|error| error.to_string());
        if let Some(problem) = problem
            .as_ref()
            .filter(|problem| reported.as_ref() != Some(*problem))
        {
            eprintln!("sparky-dock: sem janelas do compositor: {problem}");
        }
        reported = problem;
        if sender.unbounded_send(Vec::new()).is_err() {
            return;
        }
        thread::sleep(RECONNECT);
    }
}

fn watch(path: &Path, sender: &UnboundedSender<Vec<Seen>>) -> Result<(), IpcError> {
    let mut asking = Client::connect_with_timeout(path, TIMEOUT)?;
    let events = Client::connect(path)?.watch_windows()?;
    for _ in std::iter::once(()).chain(events.map(drop)) {
        if sender.unbounded_send(listed(&mut asking)?).is_err() {
            return Ok(());
        }
    }
    Ok(())
}

fn listed(client: &mut Client) -> Result<Vec<Seen>, IpcError> {
    match client.request(&Request::Windows)? {
        Reply::Windows { windows } => Ok(windows
            .into_iter()
            .map(|info| Seen {
                app_id: info.app_id,
                focused: info.focused,
            })
            .collect()),
        _ => Err(IpcError::Malformed(
            "o compositor não listou as janelas".to_owned(),
        )),
    }
}
