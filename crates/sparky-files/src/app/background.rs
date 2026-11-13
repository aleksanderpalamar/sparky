use std::thread;

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};

pub fn background<T: Send + 'static>(
    work: impl FnOnce(&dyn Fn(T)) + Send + 'static,
) -> impl Stream<Item = T> {
    iced::stream::channel(16, async move |mut output| {
        let (sender, mut events) = mpsc::unbounded();
        thread::spawn(move || {
            work(&|event| {
                let _ = sender.unbounded_send(event);
            })
        });
        while let Some(event) = events.next().await {
            if output.send(event).await.is_err() {
                return;
            }
        }
    })
}
