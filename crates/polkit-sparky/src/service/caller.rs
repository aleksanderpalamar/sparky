use zbus::{Connection, message::Header};

use super::{AUTHORITY, Error};

pub async fn verify(connection: &Connection, header: &Header<'_>) -> Result<(), Error> {
    let reply = connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "GetNameOwner",
            &(AUTHORITY,),
        )
        .await?;
    let owner: String = reply.body().deserialize()?;
    if header.sender().map(|sender| sender.as_str()) != Some(owner.as_str()) {
        return Err(Error::Failed(
            "Somente a autoridade polkit pode pedir autenticação".to_owned(),
        ));
    }
    Ok(())
}
