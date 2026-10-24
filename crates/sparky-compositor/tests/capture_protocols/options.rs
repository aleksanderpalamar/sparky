use super::{client::Client, events::manager, fixture::Error};

#[test]
fn opcoes_desconhecidas_recebem_erro_do_manager() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let _session = client.manager.create_session(
        &client.source,
        manager::Options::from_bits_retain(2),
        &client.queue.handle(),
        (),
    );
    assert!(client.roundtrip().is_err());
    assert_eq!(
        client.protocol_error(),
        Some((
            manager::Error::InvalidOption as u32,
            "ext_image_copy_capture_manager_v1".into()
        ))
    );
    Ok(())
}
