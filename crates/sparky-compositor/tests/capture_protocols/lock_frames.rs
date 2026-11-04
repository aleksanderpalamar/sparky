use wayland_client::protocol::wl_compositor;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::session_lock::v1::client::{
    ext_session_lock_manager_v1 as lock_manager, ext_session_lock_surface_v1 as lock_surface,
    ext_session_lock_v1 as lock,
};

use super::client::Client;
use super::events::Events;
use super::fixture::Error;

#[test]
fn a_superficie_de_bloqueio_recebe_o_aviso_de_quadro_para_redesenhar() -> Result<(), Error> {
    let mut client = Client::connect("frames")?;
    let qh = client.queue.handle();
    let compositor: wl_compositor::WlCompositor = client.bindings.bind(&qh, 1..=4, ())?;
    let manager: lock_manager::ExtSessionLockManagerV1 = client.bindings.bind(&qh, 1..=1, ())?;
    let session = manager.lock(&qh, ());
    client.await_event(|events| events.locked)?;
    let surface = compositor.create_surface(&qh, ());
    let lock_surface = session.get_lock_surface(&surface, &client.output, &qh, ());
    client.await_event(|events| events.lock_configure.is_some())?;
    if let Some(serial) = client.events.lock_configure {
        lock_surface.ack_configure(serial);
    }
    surface.attach(Some(&client.shm_buffer()?), 0, 0);
    surface.frame(&qh, ());
    surface.commit();
    client.await_event(|events| events.callbacks > 0)?;
    assert_eq!(client.protocol_error(), None);
    Ok(())
}

impl Dispatch<lock::ExtSessionLockV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &lock::ExtSessionLockV1,
        event: lock::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let lock::Event::Locked = event {
            state.locked = true;
        }
    }
}

impl Dispatch<lock_surface::ExtSessionLockSurfaceV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &lock_surface::ExtSessionLockSurfaceV1,
        event: lock_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let lock_surface::Event::Configure { serial, .. } = event {
            state.lock_configure = Some(serial);
        }
    }
}

wayland_client::delegate_noop!(Events: ignore lock_manager::ExtSessionLockManagerV1);
