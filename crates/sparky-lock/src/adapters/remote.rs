use std::io;

use iced::futures::StreamExt;
use iced::futures::stream::{self, BoxStream};
use tokio::signal::unix::{SignalKind, signal};

pub fn requests() -> io::Result<BoxStream<'static, ()>> {
    let listener = signal(SignalKind::user_defined1())?;
    Ok(stream::unfold(listener, |mut listener| async move {
        listener.recv().await.map(|()| ((), listener))
    })
    .boxed())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use iced::futures::StreamExt;
    use nix::sys::signal::{Signal, raise};

    use super::requests;

    #[tokio::test]
    async fn o_sigusr1_vira_um_pedido_de_desbloqueio() {
        let mut pedidos = requests().unwrap_or_else(|error| panic!("{error}"));
        raise(Signal::SIGUSR1).unwrap_or_else(|error| panic!("{error}"));
        let pedido = tokio::time::timeout(Duration::from_secs(2), pedidos.next()).await;
        assert_eq!(pedido.ok().flatten(), Some(()));
    }
}
