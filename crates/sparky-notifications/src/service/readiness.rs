use super::{PRIVATE_NAME, PUBLIC_NAME};
use std::time::Duration;

pub fn wait() -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(5), ready())
            .await
            .map_err(|_| "O serviço nativo de notificações não ficou pronto".to_owned())?
            .map_err(|error| error.to_string())
    })
}

async fn ready() -> zbus::Result<()> {
    let connection = zbus::Connection::session().await?;
    let proxy = zbus::fdo::DBusProxy::new(&connection).await?;
    loop {
        if owned(&proxy).await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

pub async fn owned(proxy: &zbus::fdo::DBusProxy<'_>) -> bool {
    let (Ok(public), Ok(private)) = (PUBLIC_NAME.try_into(), PRIVATE_NAME.try_into()) else {
        return false;
    };
    match (
        proxy.get_name_owner(public).await,
        proxy.get_name_owner(private).await,
    ) {
        (Ok(public), Ok(private)) => public == private,
        _ => false,
    }
}
