use wayland_client::globals::{GlobalList, registry_queue_init};
use wayland_client::protocol::{wl_compositor, wl_output, wl_surface};
use wayland_client::{Connection, EventQueue};

use super::events::{Events, Image, image, manager};
use super::fixture::{Error, NAME, Server};
use super::server::wait;

pub struct Client {
    connection: Connection,
    pub queue: EventQueue<Events>,
    pub events: Events,
    pub manager: manager::WpColorManagerV1,
    pub output: wl_output::WlOutput,
    pub compositor: wl_compositor::WlCompositor,
    pub globals: GlobalList,
    server: Server,
}

impl Client {
    pub fn connect() -> Result<Self, Error> {
        Self::with_version(2)
    }

    pub fn with_version(version: u32) -> Result<Self, Error> {
        let (server, connection) = Server::connect(NAME, "")?;
        let (globals, queue) = registry_queue_init::<Events>(&connection)?;
        let qh = queue.handle();
        let output = globals.bind(&qh, 2..=4, ())?;
        let manager = globals.bind(&qh, version..=version, ())?;
        let compositor = globals.bind(&qh, 1..=6, ())?;
        let mut client = Self {
            connection,
            queue,
            events: Events::default(),
            manager,
            output,
            compositor,
            globals,
            server,
        };
        client.roundtrip()?;
        Ok(client)
    }

    pub fn roundtrip(&mut self) -> Result<(), Error> {
        self.queue.roundtrip(&mut self.events)?;
        Ok(())
    }

    pub fn surface(&self) -> wl_surface::WlSurface {
        self.compositor.create_surface(&self.queue.handle(), ())
    }

    pub fn output_image(&mut self) -> Result<(image::WpImageDescriptionV1, u64), Error> {
        let qh = self.queue.handle();
        let output = self.manager.get_output(&self.output, &qh, ());
        let image = output.get_image_description(&qh, ());
        self.ready(&image).map(|identity| (image, identity))
    }

    pub fn ready(&mut self, image: &image::WpImageDescriptionV1) -> Result<u64, Error> {
        self.roundtrip()?;
        match self.events.image(image) {
            Some(Image::Ready(identity)) => Ok(identity),
            other => Err(format!("descrição não ficou pronta: {other:?}").into()),
        }
    }

    pub fn information(&mut self, image: &image::WpImageDescriptionV1) -> Result<(), Error> {
        self.events.info = Default::default();
        image.get_information(&self.queue.handle(), ());
        self.await_event(|events| events.info.done)
    }

    pub fn protocol_error(&mut self) -> Option<(u32, String)> {
        let _ = self.queue.roundtrip(&mut self.events);
        self.connection
            .protocol_error()
            .map(|error| (error.code, error.object_interface))
    }

    pub fn signal(&self, name: &str) -> Result<(), Error> {
        std::fs::write(self.server.file(name), "")?;
        Ok(())
    }

    pub fn conversions(&mut self) -> Result<Vec<String>, Error> {
        self.roundtrip()?;
        let file = self.server.file("conversions");
        self.signal("probe")?;
        let started = std::time::Instant::now();
        while started.elapsed() < std::time::Duration::from_secs(3) {
            if let Ok(text) = std::fs::read_to_string(&file) {
                std::fs::remove_file(&file)?;
                return Ok(text.lines().map(str::to_owned).collect());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Err("sonda de conversões não respondeu".into())
    }

    pub fn await_event(&mut self, ready: impl Fn(&Events) -> bool) -> Result<(), Error> {
        wait(
            &mut self.queue,
            &mut self.events,
            ready,
            "evento de cor não recebido",
        )
    }
}
