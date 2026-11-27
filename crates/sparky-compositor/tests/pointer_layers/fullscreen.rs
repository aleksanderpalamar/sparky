use wayland_client::protocol::wl_pointer::ButtonState;

use super::client::Client;
use super::fixture::Error;

#[test]
fn topo_da_tela_cheia_recebe_movimento_e_clique() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    client.send("fullscreen")?;
    client.send("move 400 300")?;
    assert_eq!(client.pointed(), Some(&client.window));
    client.send("move 400 0")?;
    assert_eq!(client.pointed(), Some(&client.window));
    client.send("press")?;
    client.send("release")?;
    let window = Some(client.window.clone());
    assert_eq!(
        client.events.presses,
        vec![
            (window.clone(), ButtonState::Pressed),
            (window, ButtonState::Released),
        ]
    );
    Ok(())
}

#[test]
fn entrar_em_tela_cheia_com_o_ponteiro_parado_no_topo() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    client.send("move 400 5")?;
    assert_eq!(client.pointed(), Some(&client.bar));
    client.send("fullscreen")?;
    assert_eq!(client.pointed(), Some(&client.window));
    Ok(())
}

#[test]
fn sair_da_tela_cheia_devolve_a_barra_ao_ponteiro_parado() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    client.send("fullscreen")?;
    client.send("move 400 5")?;
    assert_eq!(client.pointed(), Some(&client.window));
    client.send("fullscreen")?;
    assert_eq!(client.pointed(), Some(&client.bar));
    Ok(())
}

#[test]
fn maximizado_mantem_a_barra_clicavel() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    client.send("maximize")?;
    client.send("move 400 5")?;
    assert_eq!(client.pointed(), Some(&client.bar));
    client.send("press")?;
    assert_eq!(
        client.events.presses,
        vec![(Some(client.bar.clone()), ButtonState::Pressed)]
    );
    Ok(())
}

#[test]
fn ponteiro_confinado_chega_ao_topo_da_tela_cheia() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    client.send("fullscreen")?;
    client.send("move 400 300")?;
    client.confine()?;
    client.send("shift 0 -298")?;
    assert_eq!(client.pointed(), Some(&client.window));
    assert!(
        client.events.last_y.is_some_and(|y| y < 5.0),
        "{:?}",
        client.events.last_y
    );
    Ok(())
}
