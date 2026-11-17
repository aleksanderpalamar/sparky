use super::proxy;
use zbus::{Connection, zvariant::ObjectPath};

pub async fn remove(
    connection: &Connection,
    adapter: &str,
    devices: &[String],
) -> zbus::Result<()> {
    let prefix = format!("{adapter}/");
    if devices.iter().any(|device| !device.starts_with(&prefix)) {
        return Err(zbus::Error::Failure(crate::i18n::text(
            "bluetooth-invalid-path",
        )));
    }
    let adapter = proxy(connection, adapter, "org.bluez.Adapter1").await?;
    for device in devices {
        adapter
            .call::<_, _, ()>("RemoveDevice", &(ObjectPath::try_from(device.as_str())?,))
            .await?;
    }
    Ok(())
}
