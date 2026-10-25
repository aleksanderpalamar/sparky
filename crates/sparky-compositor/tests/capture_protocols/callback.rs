use super::{
    client::Client,
    events::{manager, window_manager},
    fixture::Error,
    window_events::list,
};
use wayland_client::protocol::{wl_callback, wl_compositor};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::xdg::shell::client::xdg_wm_base;

impl Dispatch<wl_callback::WlCallback, ()> for super::events::Events {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.callbacks += 1;
        }
    }
}

#[test]
fn submissao_bem_sucedida_libera_callback_de_janela_em_outro_workspace() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let qh = client.queue.handle();
    let compositor: wl_compositor::WlCompositor = client.bindings.bind(&qh, 1..=4, ())?;
    let shell: xdg_wm_base::XdgWmBase = client.bindings.bind(&qh, 1..=6, ())?;
    let surface = compositor.create_surface(&qh, ());
    let xdg = shell.get_xdg_surface(&surface, &qh, ());
    let _window = xdg.get_toplevel(&qh, ());
    surface.commit();
    client.roundtrip()?;
    client.roundtrip()?;
    xdg.set_window_geometry(0, 0, 320, 240);
    surface.attach(Some(&client.shm_buffer()?), 0, 0);
    surface.commit();
    client.roundtrip()?;
    let _list: list::ExtForeignToplevelListV1 = client.bindings.bind(&qh, 1..=1, ())?;
    client.await_event(|events| !events.windows.is_empty())?;
    let sources: window_manager::ExtForeignToplevelImageCaptureSourceManagerV1 =
        client.bindings.bind(&qh, 1..=1, ())?;
    let source = sources.create_source(&client.events.windows[0], &qh, ());
    let session = client
        .manager
        .create_session(&source, manager::Options::empty(), &qh, ());
    client.await_event(|events| events.size == Some((320, 240)))?;
    let _frame = session.create_frame(&qh, ());
    let _callback = surface.frame(&qh, ());
    surface.commit();
    client.roundtrip()?;
    assert_eq!(client.events.callbacks, 0);
    client.signal("submit-hidden")?;
    client.await_event(|events| events.ready == 1)?;
    assert_eq!(client.events.callbacks, 1);
    assert!(client.events.failures.is_empty());
    Ok(())
}
