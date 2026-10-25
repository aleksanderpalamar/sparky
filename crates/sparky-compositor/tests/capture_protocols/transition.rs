use super::{client::Client, fixture::Error};

#[test]
fn bloquear_e_desbloquear_preserva_a_sessao_e_o_proximo_quadro() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    for (index, signal) in ["lock", "unlock", "lock", "unlock"].into_iter().enumerate() {
        let frame = client.frame();
        client.roundtrip()?;
        client.signal(signal)?;
        client.await_event(|events| events.ready == index + 1)?;
        assert!(client.events.failures.is_empty());
        assert_eq!(client.events.stopped, 0);
        frame.destroy();
        client.roundtrip()?;
    }
    Ok(())
}
