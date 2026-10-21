use std::fs::File;
use std::io::Read;

use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::wp::linux_dmabuf::zv1::client::{
    zwp_linux_dmabuf_feedback_v1 as feedback, zwp_linux_dmabuf_v1 as dmabuf,
};

use super::test_server::{Error, Server};

#[derive(Default)]
pub struct Events {
    pub device: Option<libc::dev_t>,
    pub targets: Vec<libc::dev_t>,
    pub table: Vec<u8>,
    pub indices: Vec<u16>,
    pub formats: Vec<(u32, u64)>,
    pub done: bool,
    pub error: Option<String>,
}

pub struct Client {
    pub advertised: u32,
    pub events: Events,
    queue: EventQueue<Events>,
    dmabuf: dmabuf::ZwpLinuxDmabufV1,
    _server: Server,
}

impl Client {
    pub fn connect(device: Option<libc::dev_t>, version: u32) -> Result<Self, Error> {
        let (server, connection) = Server::connect(device)?;
        let (globals, mut queue) = registry_queue_init::<Events>(&connection)?;
        let advertised = globals
            .contents()
            .clone_list()
            .into_iter()
            .find(|global| global.interface == "zwp_linux_dmabuf_v1")
            .ok_or("protocolo DMA-BUF ausente")?
            .version;
        let version = version.min(advertised);
        let dmabuf = globals.bind(&queue.handle(), version..=version, ())?;
        let mut events = Events::default();
        queue.roundtrip(&mut events)?;
        Ok(Self {
            advertised,
            events,
            queue,
            dmabuf,
            _server: server,
        })
    }

    pub fn feedback(&mut self) -> Result<(), Error> {
        let _feedback = self.dmabuf.get_default_feedback(&self.queue.handle(), ());
        self.queue.roundtrip(&mut self.events)?;
        Ok(())
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Events {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<dmabuf::ZwpLinuxDmabufV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &dmabuf::ZwpLinuxDmabufV1,
        event: dmabuf::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let dmabuf::Event::Modifier {
            format,
            modifier_hi,
            modifier_lo,
        } = event
        {
            state
                .formats
                .push((format, (u64::from(modifier_hi) << 32) | u64::from(modifier_lo)));
        }
    }
}

impl Dispatch<feedback::ZwpLinuxDmabufFeedbackV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &feedback::ZwpLinuxDmabufFeedbackV1,
        event: feedback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            feedback::Event::MainDevice { device } => state.device = device_id(&device),
            feedback::Event::TrancheTargetDevice { device } => state.targets.extend(device_id(&device)),
            feedback::Event::TrancheFormats { indices } => {
                state.indices.extend(
                    indices
                        .chunks_exact(2)
                        .map(|bytes| u16::from_ne_bytes([bytes[0], bytes[1]])),
                );
            }
            feedback::Event::FormatTable { fd, size } => {
                let result = File::from(fd).take(u64::from(size)).read_to_end(&mut state.table);
                state.error = result.err().map(|error| error.to_string());
            }
            feedback::Event::Done => state.done = true,
            _ => {}
        }
    }
}

fn device_id(bytes: &[u8]) -> Option<libc::dev_t> {
    bytes.try_into().ok().map(libc::dev_t::from_ne_bytes)
}
