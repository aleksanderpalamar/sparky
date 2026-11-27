use std::{fs, thread, time::Duration, time::Instant};

use wayland_client::EventQueue;
use wayland_client::globals::{GlobalList, registry_queue_init};
use wayland_client::protocol::{wl_compositor, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface};

use super::events::{Events, layer_shell, xdg_wm_base};
use super::fixture::{COMMAND, Error, NAME, Server};
use super::surfaces::Bar;
use super::windows::{Toplevel, toplevel};

pub struct Client {
    pub events: Events,
    pub queue: EventQueue<Events>,
    pub globals: GlobalList,
    pub compositor: wl_compositor::WlCompositor,
    pub shm: wl_shm::WlShm,
    pub layers: layer_shell::ZwlrLayerShellV1,
    pub window: wl_surface::WlSurface,
    pub bar: wl_surface::WlSurface,
    pub first: Toplevel,
    pub base: xdg_wm_base::XdgWmBase,
    pub seat: wl_seat::WlSeat,
    pub pointer: wl_pointer::WlPointer,
    server: Server,
}

impl Client {
    pub fn connect(scenario: &str) -> Result<Self, Error> {
        let (server, connection) = Server::connect(NAME, scenario)?;
        let (globals, mut queue) = registry_queue_init::<Events>(&connection)?;
        let qh = queue.handle();
        let compositor: wl_compositor::WlCompositor = globals.bind(&qh, 1..=6, ())?;
        let shm: wl_shm::WlShm = globals.bind(&qh, 1..=1, ())?;
        let base: xdg_wm_base::XdgWmBase = globals.bind(&qh, 1..=5, ())?;
        let layers: layer_shell::ZwlrLayerShellV1 = globals.bind(&qh, 1..=4, ())?;
        let seat: wl_seat::WlSeat = globals.bind(&qh, 1..=7, ())?;
        let pointer = seat.get_pointer(&qh, ());
        seat.get_keyboard(&qh, ());
        let mut events = Events::default();
        let first = toplevel(&mut queue, &mut events, &compositor, &base, &shm)?;
        let mut client = Self {
            events,
            queue,
            globals,
            compositor,
            shm,
            layers,
            window: first.surface.clone(),
            bar: first.surface.clone(),
            first,
            base,
            seat,
            pointer,
            server,
        };
        client.bar = client.layer(&Bar::top(), None)?;
        Ok(client)
    }

    pub fn output(&self, index: usize) -> Option<wl_output::WlOutput> {
        let qh = self.queue.handle();
        self.globals
            .contents()
            .clone_list()
            .into_iter()
            .filter(|global| global.interface == "wl_output")
            .nth(index)
            .map(|global| self.globals.registry().bind(global.name, 4, &qh, ()))
    }

    pub fn send(&mut self, command: &str) -> Result<(), Error> {
        let path = self.server.file(COMMAND);
        let pending = self.server.file("command.tmp");
        fs::write(&pending, command)?;
        fs::rename(pending, &path)?;
        let started = Instant::now();
        while path.exists() && started.elapsed() < Duration::from_secs(3) {
            thread::sleep(Duration::from_millis(5));
        }
        self.queue.roundtrip(&mut self.events)?;
        self.queue.roundtrip(&mut self.events)?;
        Ok(())
    }

    pub fn pointed(&self) -> Option<&wl_surface::WlSurface> {
        self.events.focus.as_ref()
    }
}
