use zbus::Proxy;

const UNAVAILABLE: &str = ".ProfileUnavailable";

pub async fn connect(device: &Proxy<'_>, name: &str) -> zbus::Result<()> {
    match device.call::<_, _, ()>("Connect", &()).await {
        Err(error) if unavailable(&error) => {
            let _ = device.call::<_, _, ()>("Disconnect", &()).await;
            Err(zbus::Error::Failure(crate::i18n::with(
                "bluetooth-no-profiles",
                "name",
                name,
            )))
        }
        other => other,
    }
}

pub fn unavailable(error: &zbus::Error) -> bool {
    match error {
        zbus::Error::MethodError(name, _, _) => name.as_str().ends_with(UNAVAILABLE),
        _ => false,
    }
}
