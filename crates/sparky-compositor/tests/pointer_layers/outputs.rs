use super::client::Client;
use super::fixture::Error;
use super::surfaces::Bar;

#[test]
fn camadas_do_monitor_removido_recebem_closed() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    client.send("unplug")?;
    assert_eq!(client.events.layers_closed, 1);
    Ok(())
}

#[test]
fn camada_nova_aparece_no_monitor_que_volta_com_o_mesmo_nome() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    client.send("unplug")?;
    client.send("plug")?;
    let dock = client.layer(&Bar::top(), None)?;
    client.send("move 400 5")?;
    assert_eq!(client.pointed(), Some(&dock));
    Ok(())
}
