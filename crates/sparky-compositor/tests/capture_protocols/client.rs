use super::{
    events::*,
    fixture::{Error, Server},
};
use std::os::fd::AsFd;
use wayland_client::globals::{GlobalList, registry_queue_init};
use wayland_client::{
    EventQueue,
    protocol::{wl_output, wl_shm},
};

pub struct Client {
    connection: wayland_client::Connection,
    pub output: wl_output::WlOutput,
    pub events: Events,
    pub queue: EventQueue<Events>,
    pub session: session::ExtImageCopyCaptureSessionV1,
    pub manager: manager::ExtImageCopyCaptureManagerV1,
    pub source: source::ExtImageCaptureSourceV1,
    pub shm: wl_shm::WlShm,
    pub globals: Vec<String>,
    pub bindings: GlobalList,
    _server: Server,
}

impl Client {
    pub fn connect(scenario: &str) -> Result<Self, Error> {
        let (server, connection) = Server::connect(scenario)?;
        let (globals, mut queue) = registry_queue_init::<Events>(&connection)?;
        let qh = queue.handle();
        let output: wl_output::WlOutput = globals.bind(&qh, 1..=4, ())?;
        let sources: output_manager::ExtOutputImageCaptureSourceManagerV1 =
            globals.bind(&qh, 1..=1, ())?;
        let manager: manager::ExtImageCopyCaptureManagerV1 = globals.bind(&qh, 1..=1, ())?;
        let shm = globals.bind(&qh, 1..=1, ())?;
        let source = sources.create_source(&output, &qh, ());
        let session = manager.create_session(&source, manager::Options::empty(), &qh, ());
        let mut events = Events::default();
        queue.roundtrip(&mut events)?;
        let names = globals
            .contents()
            .clone_list()
            .into_iter()
            .map(|global| global.interface)
            .collect();
        Ok(Self {
            connection,
            output,
            events,
            queue,
            session,
            manager,
            source,
            shm,
            globals: names,
            bindings: globals,
            _server: server,
        })
    }

    pub fn frame(&self) -> frame::ExtImageCopyCaptureFrameV1 {
        self.session.create_frame(&self.queue.handle(), ())
    }

    pub fn protocol_error(&self) -> Option<(u32, String)> {
        self.connection
            .protocol_error()
            .map(|error| (error.code, error.object_interface))
    }

    pub fn signal(&self, name: &str) -> Result<(), Error> {
        self._server.signal(name)?;
        Ok(())
    }

    pub fn await_event(&mut self, ready: impl Fn(&Events) -> bool) -> Result<(), Error> {
        let started = std::time::Instant::now();
        while started.elapsed() < std::time::Duration::from_secs(3) {
            self.roundtrip()?;
            if ready(&self.events) {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Err("evento de captura não recebido".into())
    }

    pub fn roundtrip(&mut self) -> Result<(), Error> {
        self.queue.roundtrip(&mut self.events)?;
        Ok(())
    }

    pub fn shm_buffer(&self) -> Result<wayland_client::protocol::wl_buffer::WlBuffer, Error> {
        let file = tempfile::tempfile()?;
        file.set_len(800 * 600 * 4)?;
        let qh = self.queue.handle();
        let pool = self.shm.create_pool(file.as_fd(), 800 * 600 * 4, &qh, ());
        let buffer = pool.create_buffer(0, 800, 600, 800 * 4, wl_shm::Format::Argb8888, &qh, ());
        pool.destroy();
        Ok(buffer)
    }
}
