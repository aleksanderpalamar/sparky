use wayland_client::EventQueue;
use wayland_client::protocol::{wl_compositor, wl_shm, wl_surface};

use super::client::Client;
use super::events::{Events, xdg_positioner, xdg_surface, xdg_toplevel, xdg_wm_base};
use super::fixture::Error;
use super::server::wait;
use super::surfaces::buffer;

pub struct Toplevel {
    pub surface: wl_surface::WlSurface,
    pub xdg: xdg_surface::XdgSurface,
    pub role: xdg_toplevel::XdgToplevel,
}

pub fn toplevel(
    queue: &mut EventQueue<Events>,
    events: &mut Events,
    compositor: &wl_compositor::WlCompositor,
    base: &xdg_wm_base::XdgWmBase,
    shm: &wl_shm::WlShm,
) -> Result<Toplevel, Error> {
    let qh = queue.handle();
    let surface = compositor.create_surface(&qh, ());
    let xdg = base.get_xdg_surface(&surface, &qh, ());
    let role = xdg.get_toplevel(&qh, ());
    surface.commit();
    let before = events.configured;
    wait(
        queue,
        events,
        |events| events.configured > before,
        "janela sem configure",
    )?;
    surface.attach(Some(&buffer(shm, &qh, 800, 600)?), 0, 0);
    surface.commit();
    queue.roundtrip(events)?;
    Ok(Toplevel { surface, xdg, role })
}

impl Client {
    pub fn open(&mut self) -> Result<Toplevel, Error> {
        toplevel(
            &mut self.queue,
            &mut self.events,
            &self.compositor,
            &self.base,
            &self.shm,
        )
    }

    pub fn popup(
        &mut self,
        parent: &xdg_surface::XdgSurface,
        area: (i32, i32, i32, i32),
    ) -> Result<wl_surface::WlSurface, Error> {
        let qh = self.queue.handle();
        let (x, y, width, height) = area;
        let positioner = self.base.create_positioner(&qh, ());
        positioner.set_size(width, height);
        positioner.set_anchor_rect(x, y, 1, 1);
        positioner.set_anchor(xdg_positioner::Anchor::TopLeft);
        positioner.set_gravity(xdg_positioner::Gravity::BottomRight);
        let surface = self.compositor.create_surface(&qh, ());
        let xdg = self.base.get_xdg_surface(&surface, &qh, ());
        xdg.get_popup(Some(parent), &positioner, &qh, ());
        surface.commit();
        let before = self.events.configured;
        wait(
            &mut self.queue,
            &mut self.events,
            |events| events.configured > before,
            "menu sem configure",
        )?;
        let size = (width as u32, height as u32);
        surface.attach(Some(&buffer(&self.shm, &qh, size.0, size.1)?), 0, 0);
        surface.commit();
        self.queue.roundtrip(&mut self.events)?;
        Ok(surface)
    }
}
