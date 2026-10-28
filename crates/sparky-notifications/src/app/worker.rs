use super::Message;
use crate::{
    client::Client,
    service::{self, Shared},
};
use iced::futures::{SinkExt, Stream};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Clone)]
pub struct Worker(pub Arc<Shared>);

impl Hash for Worker {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

pub fn events(worker: &Worker) -> impl Stream<Item = Message> + use<> {
    let shared = worker.0.clone();
    iced::stream::channel(8, async move |mut output| {
        let result = serve(shared, &mut output).await;
        if let Err(error) = result {
            let _ = output.send(Message::Fatal(error.to_string())).await;
        }
        std::future::pending::<()>().await;
    })
}

async fn serve(
    shared: Arc<Shared>,
    output: &mut iced::futures::channel::mpsc::Sender<Message>,
) -> zbus::Result<()> {
    let connection = zbus::connection::Builder::session()?
        .method_timeout(std::time::Duration::from_secs(3))
        .build()
        .await?;
    service::serve(&connection, shared.clone()).await?;
    if output
        .send(Message::Ready(Arc::new(Client::on(connection.clone()))))
        .await
        .is_err()
    {
        return Ok(());
    }
    let mut changes = output.clone();
    service::pump(&connection, shared, move || {
        let _ = changes.try_send(Message::Changed);
        !changes.is_closed()
    })
    .await
}
