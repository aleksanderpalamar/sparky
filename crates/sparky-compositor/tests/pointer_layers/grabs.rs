use super::client::Client;
use super::events::xdg_toplevel::ResizeEdge;
use super::fixture::Error;

fn grab(client: &mut Client) -> Result<u32, Error> {
    client.send("move 200 300")?;
    client.send("press")?;
    client
        .events
        .serial
        .ok_or_else(|| "botão sem serial".into())
}

#[test]
fn redimensionar_pelo_mouse_nao_trava_o_compositor() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    client.open()?;
    let serial = grab(&mut client)?;
    client
        .first
        .role
        .resize(&client.seat, serial, ResizeEdge::BottomRight);
    client.queue.roundtrip(&mut client.events)?;
    client.send("move 260 360")?;
    client.send("release")?;
    client.send("move 400 300")?;
    assert_eq!(client.pointed(), Some(&client.window));
    Ok(())
}

#[test]
fn trocar_janelas_pelo_mouse_nao_trava_o_compositor() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    let second = client.open()?;
    let serial = grab(&mut client)?;
    client.first.role._move(&client.seat, serial);
    client.queue.roundtrip(&mut client.events)?;
    client.send("move 600 300")?;
    client.send("release")?;
    client.send("move 200 300")?;
    assert_eq!(client.pointed(), Some(&second.surface));
    Ok(())
}

#[test]
fn menu_de_janela_sem_foco_ativa_a_janela_dona() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    let parent = client.first.xdg.clone();
    let second = client.open()?;
    client.popup(&parent, (100, 100, 120, 120))?;
    client.send("move 600 300")?;
    assert_eq!(client.events.keyboard.as_ref(), Some(&second.surface));
    client.send("move 170 190")?;
    assert_eq!(client.events.keyboard.as_ref(), Some(&client.window));
    Ok(())
}
