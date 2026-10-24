use super::{
    client::Client,
    events::{manager, window_manager},
    fixture::Error,
    window_events::list,
};
use wayland_client::protocol::wl_compositor;
use wayland_protocols::xdg::shell::client::xdg_wm_base;

#[test]
fn redimensionar_reanuncia_as_restricoes_e_remover_monitor_para_a_sessao() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    client.signal("resize")?;
    client.await_event(|events| events.size == Some((640, 480)))?;
    assert_eq!(client.events.done, 2);
    client.signal("remove")?;
    client.await_event(|events| events.stopped == 1)?;
    Ok(())
}

#[test]
fn janela_real_e_listada_com_tamanho_proprio_e_sessao_encerra_ao_fechar() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let qh = client.queue.handle();
    let compositor: wl_compositor::WlCompositor = client.bindings.bind(&qh, 1..=4, ())?;
    let shell: xdg_wm_base::XdgWmBase = client.bindings.bind(&qh, 1..=6, ())?;
    let surface = compositor.create_surface(&qh, ());
    let xdg = shell.get_xdg_surface(&surface, &qh, ());
    let window = xdg.get_toplevel(&qh, ());
    window.set_title("Janela privada de teste".into());
    surface.commit();
    client.roundtrip()?;
    client.roundtrip()?;
    xdg.set_window_geometry(0, 0, 320, 240);
    surface.attach(Some(&client.shm_buffer()?), 0, 0);
    surface.commit();
    client.roundtrip()?;
    let _list: list::ExtForeignToplevelListV1 = client.bindings.bind(&qh, 1..=1, ())?;
    client.await_event(|events| !events.windows.is_empty() && !events.titles.is_empty())?;
    assert_eq!(client.events.titles[0], "Janela privada de teste");
    let sources: window_manager::ExtForeignToplevelImageCaptureSourceManagerV1 =
        client.bindings.bind(&qh, 1..=1, ())?;
    let source = sources.create_source(&client.events.windows[0], &qh, ());
    let session = client
        .manager
        .create_session(&source, manager::Options::empty(), &qh, ());
    client.await_event(|events| events.size == Some((320, 240)))?;
    assert_eq!(client.events.stopped, 0);
    window.destroy();
    xdg.destroy();
    surface.destroy();
    client.await_event(|events| events.stopped == 1 && events.closed == 1)?;
    session.destroy();
    Ok(())
}
