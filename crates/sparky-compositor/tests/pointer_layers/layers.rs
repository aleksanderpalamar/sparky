use super::client::Client;
use super::fixture::Error;
use super::surfaces::Bar;

#[test]
fn overlay_segue_acima_da_tela_cheia_so_na_regiao_de_entrada() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    let overlay = client.layer(&Bar::overlay((0, 0, 50, 50)), None)?;
    client.send("fullscreen")?;
    client.send("move 20 20")?;
    assert_eq!(client.pointed(), Some(&overlay));
    client.send("move 80 80")?;
    assert_eq!(client.pointed(), Some(&client.window));
    Ok(())
}

#[test]
fn sessao_bloqueada_nao_entrega_o_ponteiro_a_ninguem() -> Result<(), Error> {
    let mut client = Client::connect("one")?;
    client.send("fullscreen")?;
    client.send("move 400 300")?;
    client.send("lock")?;
    client.send("move 400 5")?;
    assert_eq!(client.pointed(), None);
    Ok(())
}

#[test]
fn barra_do_outro_monitor_segue_acessivel() -> Result<(), Error> {
    let mut client = Client::connect("two")?;
    let output = client.output(1).ok_or("segundo monitor ausente")?;
    let second = client.layer(&Bar::top(), Some(&output))?;
    client.send("fullscreen")?;
    client.send("move 1200 5")?;
    assert_eq!(client.pointed(), Some(&second));
    client.send("move 400 5")?;
    assert_eq!(client.pointed(), Some(&client.window));
    Ok(())
}
