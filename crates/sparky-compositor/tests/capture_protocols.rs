#[path = "capture_protocols/callback.rs"]
mod callback;
#[path = "capture_protocols/client.rs"]
mod client;
#[path = "capture_protocols/events.rs"]
mod events;
#[path = "capture_protocols/fixture.rs"]
mod fixture;
#[path = "capture_protocols/legacy.rs"]
mod legacy;
#[path = "capture_protocols/lifecycle.rs"]
mod lifecycle;
#[path = "capture_protocols/options.rs"]
mod options;
#[path = "capture_protocols/submission.rs"]
mod submission;
#[path = "capture_protocols/transition.rs"]
mod transition;
#[path = "capture_protocols/window_events.rs"]
mod window_events;

use client::Client;
use events::{frame, manager};
use fixture::Error;
use wayland_client::WEnum;

#[test]
fn oferece_monitor_janela_e_lista_de_janelas() -> Result<(), Error> {
    let client = Client::connect("")?;
    for interface in [
        "ext_image_copy_capture_manager_v1",
        "ext_output_image_capture_source_manager_v1",
        "ext_foreign_toplevel_image_capture_source_manager_v1",
        "ext_foreign_toplevel_list_v1",
    ] {
        assert!(
            client.globals.iter().any(|name| name == interface),
            "{interface}"
        );
    }
    Ok(())
}

#[test]
fn anuncia_tamanho_dispositivo_e_modificadores_antes_do_done() -> Result<(), Error> {
    let client = Client::connect("")?;
    assert_eq!(client.events.size, Some((800, 600)));
    assert_eq!(
        client.events.device,
        Some(0x1234_u64.to_ne_bytes().to_vec())
    );
    assert_eq!(client.events.formats.len(), 2);
    let argb = smithay::backend::allocator::Fourcc::Argb8888 as u32;
    assert_eq!(
        client
            .events
            .formats
            .iter()
            .find(|(code, _)| *code == argb)
            .map(|(_, bytes)| bytes.len()),
        Some(16)
    );
    assert_eq!(client.events.done, 1);
    assert_eq!(client.events.stopped, 0);
    Ok(())
}

#[test]
fn sessao_bloqueada_ou_sem_formato_nao_libera_captura() -> Result<(), Error> {
    for scenario in ["locked", "no-formats"] {
        let client = Client::connect(scenario)?;
        assert_eq!(client.events.stopped, 1);
        assert_eq!(client.events.done, 0);
    }
    Ok(())
}

#[test]
fn rejeita_dois_quadros_sem_destruir_o_anterior() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let _first = client.frame();
    let _second = client.frame();
    assert!(client.roundtrip().is_err());
    assert_eq!(
        client.protocol_error(),
        Some((1, "ext_image_copy_capture_session_v1".into()))
    );
    Ok(())
}

#[test]
fn destruir_quadro_permite_criar_outro() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    client.frame().destroy();
    let _second = client.frame();
    client.roundtrip()?;
    Ok(())
}

#[test]
fn capture_sem_buffer_produz_erro_de_protocolo() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    client.frame().capture();
    assert!(client.roundtrip().is_err());
    assert_eq!(
        client.protocol_error(),
        Some((1, "ext_image_copy_capture_frame_v1".into()))
    );
    Ok(())
}

#[test]
fn dano_invalido_produz_erro_de_protocolo() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    client.frame().damage_buffer(-1, 0, 1, 1);
    assert!(client.roundtrip().is_err());
    assert_eq!(
        client.protocol_error(),
        Some((2, "ext_image_copy_capture_frame_v1".into()))
    );
    Ok(())
}

#[test]
fn dano_fora_da_fonte_e_permitido_e_shm_e_recusado_sem_readback() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let frame = client.frame();
    frame.damage_buffer(1000, 1000, 100, 100);
    frame.attach_buffer(&client.shm_buffer()?);
    frame.capture();
    client.roundtrip()?;
    assert_eq!(
        client.events.failures,
        [WEnum::Value(frame::FailureReason::BufferConstraints)]
    );
    assert_eq!(client.events.ready, 0);
    Ok(())
}

#[test]
fn destruir_sessao_preserva_o_quadro_criado() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let frame = client.frame();
    client.session.destroy();
    frame.attach_buffer(&client.shm_buffer()?);
    frame.capture();
    client.roundtrip()?;
    assert_eq!(
        client.events.failures,
        [WEnum::Value(frame::FailureReason::BufferConstraints)]
    );
    Ok(())
}

#[test]
fn destruir_fonte_nao_invalida_a_sessao() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    client.source.destroy();
    let _frame = client.frame();
    client.roundtrip()?;
    Ok(())
}

#[test]
fn limita_sessoes_ativas_e_recupera_apos_destruicao() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let qh = client.queue.handle();
    let sessions: Vec<_> = (0..16)
        .map(|_| {
            client
                .manager
                .create_session(&client.source, manager::Options::empty(), &qh, ())
        })
        .collect();
    client.roundtrip()?;
    assert_eq!(client.events.done, 16);
    assert_eq!(client.events.stopped, 1);
    sessions[0].destroy();
    let _next = client
        .manager
        .create_session(&client.source, manager::Options::empty(), &qh, ());
    client.roundtrip()?;
    assert_eq!(client.events.done, 17);
    Ok(())
}
