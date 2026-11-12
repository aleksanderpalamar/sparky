use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use iced::event::{self, Event, Status};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{Subscription, time, window};
use iced::{keyboard, mouse};
use sparky_ui::press::{Press, press};

use crate::adapters::watch::watch;
use crate::app::message::Message;
use crate::app::state::Files;

const POLL: Duration = Duration::from_secs(2);

#[derive(Hash)]
struct Watched(PathBuf);

pub fn subscription(files: &Files) -> Subscription<Message> {
    Subscription::batch([
        sparky_ui::palette::subscription().map(Message::Palette),
        event::listen_with(keyboard_message),
        window::resize_events().map(|(_, size)| Message::Resized(size)),
        time::every(POLL).map(|_| Message::Tick),
        folder(files),
    ])
}

fn folder(files: &Files) -> Subscription<Message> {
    match files.sidebar_page.is_none() && files.network_uri.is_none() {
        true => Subscription::run_with(Watched(files.location.clone()), changes),
        false => Subscription::none(),
    }
}

fn changes(watched: &Watched) -> impl Stream<Item = Message> + use<> {
    let dir = watched.0.clone();
    iced::stream::channel(1, async move |mut output| {
        let (sender, mut changed) = mpsc::channel(1);
        thread::spawn(move || forward(&dir, sender));
        while changed.next().await.is_some() {
            if output.send(Message::FolderChanged).await.is_err() {
                return;
            }
        }
    })
}

fn forward(dir: &Path, mut sender: mpsc::Sender<()>) {
    let alive = sender.clone();
    let _ = watch(
        dir,
        || !alive.is_closed(),
        || !matches!(sender.try_send(()), Err(error) if error.is_disconnected()),
    );
}

fn keyboard_message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            return Some(Message::Pointer(position));
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            return Some(Message::Released);
        }
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            return Some(Message::Modifiers(modifiers));
        }
        _ => {}
    }
    press(event, status).map(|press| match press {
        Press::Escape => Message::Escape,
        Press::Key(key, modifiers) => Message::Key(key, modifiers),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use iced::futures::StreamExt;
    use iced::futures::executor::block_on;

    use super::{Watched, changes};
    use crate::app::message::Message;

    #[test]
    fn mudanca_na_pasta_vira_mensagem_para_o_aplicativo() {
        let dir = tempfile::tempdir().unwrap();
        let watched = Watched(dir.path().to_path_buf());
        let (envio, recebido) = mpsc::channel();
        thread::spawn(move || {
            let mut stream = Box::pin(changes(&watched));
            let _ = envio.send(block_on(stream.next()));
        });
        thread::sleep(Duration::from_millis(200));
        fs::write(dir.path().join("novo.txt"), "x").unwrap();
        let mensagem = recebido.recv_timeout(Duration::from_secs(3)).unwrap();
        assert!(matches!(mensagem, Some(Message::FolderChanged)));
    }
}
