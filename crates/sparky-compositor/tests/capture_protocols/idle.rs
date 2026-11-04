use std::thread;
use std::time::Duration;

use sparky_ipc::{IdleHold, Reply, Request};
use wayland_client::protocol::{wl_compositor, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1 as notification, ext_idle_notifier_v1 as notifier,
};
use wayland_protocols::ext::session_lock::v1::client::ext_session_lock_manager_v1 as lock_manager;
use wayland_protocols::wp::idle_inhibit::zv1::client::{
    zwp_idle_inhibit_manager_v1 as inhibit_manager, zwp_idle_inhibitor_v1 as inhibitor,
};

use super::client::Client;
use super::events::Events;
use super::fixture::Error;

const PARADO_MS: u32 = 50;

fn avisar_quando_parado(client: &mut Client) -> Result<notification::ExtIdleNotificationV1, Error> {
    let qh = client.queue.handle();
    let seat: wl_seat::WlSeat = client.bindings.bind(&qh, 1..=7, ())?;
    let notifier: notifier::ExtIdleNotifierV1 = client.bindings.bind(&qh, 1..=1, ())?;
    Ok(notifier.get_idle_notification(PARADO_MS, &seat, &qh, ()))
}

#[test]
fn quem_pede_e_avisado_quando_a_sessao_fica_parada() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let _aviso = avisar_quando_parado(&mut client)?;
    client.await_event(|events| events.idled > 0)?;
    Ok(())
}

#[test]
fn um_inibidor_vivo_segura_o_aviso_ate_ser_destruido() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let qh = client.queue.handle();
    let compositor: wl_compositor::WlCompositor = client.bindings.bind(&qh, 1..=4, ())?;
    let manager: inhibit_manager::ZwpIdleInhibitManagerV1 = client.bindings.bind(&qh, 1..=1, ())?;
    let surface = compositor.create_surface(&qh, ());
    let inibidor = manager.create_inhibitor(&surface, &qh, ());
    client.roundtrip()?;
    let _aviso = avisar_quando_parado(&mut client)?;
    for _ in 0..20 {
        client.roundtrip()?;
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(client.events.idled, 0);
    inibidor.destroy();
    client.await_event(|events| events.idled > 0)?;
    Ok(())
}

fn sem_aviso_por_um_tempo(client: &mut Client) -> Result<(), Error> {
    for _ in 0..20 {
        client.roundtrip()?;
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(client.events.idled, 0);
    Ok(())
}

#[test]
fn o_pedido_do_guarda_pelo_ipc_segura_o_aviso_enquanto_a_conexao_vive() -> Result<(), Error> {
    let mut client = Client::connect("ipc")?;
    let mut guarda = client.ipc()?;
    let pedido = |hold| Request::Idle { hold };
    assert_eq!(guarda.request(&pedido(IdleHold::Held))?, Reply::Done);
    let _aviso = avisar_quando_parado(&mut client)?;
    sem_aviso_por_um_tempo(&mut client)?;
    assert_eq!(guarda.request(&pedido(IdleHold::Free))?, Reply::Done);
    client.await_event(|events| events.idled > 0)?;
    Ok(())
}

#[test]
fn a_trava_cai_quando_a_conexao_do_guarda_fecha() -> Result<(), Error> {
    let mut client = Client::connect("ipc")?;
    let mut guarda = client.ipc()?;
    let pedido = Request::Idle {
        hold: IdleHold::Held,
    };
    assert_eq!(guarda.request(&pedido)?, Reply::Done);
    let _aviso = avisar_quando_parado(&mut client)?;
    sem_aviso_por_um_tempo(&mut client)?;
    drop(guarda);
    client.await_event(|events| events.idled > 0)?;
    Ok(())
}

#[test]
fn desbloquear_conta_como_atividade() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let qh = client.queue.handle();
    let manager: lock_manager::ExtSessionLockManagerV1 = client.bindings.bind(&qh, 1..=1, ())?;
    let _aviso = avisar_quando_parado(&mut client)?;
    let session = manager.lock(&qh, ());
    client.await_event(|events| events.locked && events.idled > 0)?;
    session.unlock_and_destroy();
    client.await_event(|events| events.resumed > 0)?;
    Ok(())
}

impl Dispatch<notification::ExtIdleNotificationV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &notification::ExtIdleNotificationV1,
        event: notification::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            notification::Event::Idled => state.idled += 1,
            notification::Event::Resumed => state.resumed += 1,
            _ => {}
        }
    }
}

wayland_client::delegate_noop!(Events: ignore wl_seat::WlSeat);
wayland_client::delegate_noop!(Events: ignore notifier::ExtIdleNotifierV1);
wayland_client::delegate_noop!(Events: ignore inhibit_manager::ZwpIdleInhibitManagerV1);
wayland_client::delegate_noop!(Events: ignore inhibitor::ZwpIdleInhibitorV1);
