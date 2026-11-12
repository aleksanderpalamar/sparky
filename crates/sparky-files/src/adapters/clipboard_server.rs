use std::fs::File;
use std::io;

use crate::adapters::clipboard_request::Content;
use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle, event_created_child};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1 as device, zwlr_data_control_manager_v1 as manager,
    zwlr_data_control_offer_v1 as offer, zwlr_data_control_source_v1 as source,
};

#[derive(Default)]
struct Server {
    seat: Option<wl_seat::WlSeat>,
    manager: Option<manager::ZwlrDataControlManagerV1>,
    entries: Vec<(String, Content)>,
    cancelled: bool,
}

pub fn serve(entries: Vec<(String, Content)>) -> io::Result<()> {
    let connection = Connection::connect_to_env().map_err(io::Error::other)?;
    let mut queue = connection.new_event_queue();
    let handle = queue.handle();
    connection.display().get_registry(&handle, ());
    let mut server = Server {
        entries,
        ..Server::default()
    };
    queue.roundtrip(&mut server).map_err(io::Error::other)?;
    let (Some(seat), Some(manager)) = (server.seat.clone(), server.manager.clone()) else {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "wlr-data-control ausente",
        ));
    };
    let device = manager.get_data_device(&seat, &handle, ());
    if server.entries.is_empty() {
        device.set_selection(None);
        queue.roundtrip(&mut server).map_err(io::Error::other)?;
        return Ok(());
    }
    let selection = manager.create_data_source(&handle, ());
    for (mime, _) in &server.entries {
        selection.offer(mime.clone());
    }
    device.set_selection(Some(&selection));
    while !server.cancelled {
        queue
            .blocking_dispatch(&mut server)
            .map_err(io::Error::other)?;
    }
    Ok(())
}

fn send(content: &Content, target: File) {
    let mut target = target;
    let _ = match content {
        Content::Text(text) => io::Write::write_all(&mut target, text.as_bytes()),
        Content::File(path) => {
            File::open(path).and_then(|mut file| io::copy(&mut file, &mut target).map(drop))
        }
    };
}

impl Dispatch<wl_registry::WlRegistry, ()> for Server {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        handle: &QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };
        match interface.as_str() {
            "wl_seat" if state.seat.is_none() => {
                state.seat = Some(registry.bind(name, version.min(1), handle, ()))
            }
            "zwlr_data_control_manager_v1" => {
                state.manager = Some(registry.bind(name, version.min(2), handle, ()));
            }
            _ => {}
        }
    }
}

impl Dispatch<source::ZwlrDataControlSourceV1, ()> for Server {
    fn event(
        state: &mut Self,
        _: &source::ZwlrDataControlSourceV1,
        event: source::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            source::Event::Send { mime_type, fd } => {
                if let Some((_, content)) =
                    state.entries.iter().find(|(mime, _)| *mime == mime_type)
                {
                    send(content, File::from(fd));
                }
            }
            source::Event::Cancelled => state.cancelled = true,
            _ => {}
        }
    }
}

impl Dispatch<device::ZwlrDataControlDeviceV1, ()> for Server {
    fn event(
        state: &mut Self,
        _: &device::ZwlrDataControlDeviceV1,
        event: device::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let device::Event::Finished = event {
            state.cancelled = true;
        }
    }

    event_created_child!(Server, device::ZwlrDataControlDeviceV1, [
        device::EVT_DATA_OFFER_OPCODE => (offer::ZwlrDataControlOfferV1, ()),
    ]);
}

impl Dispatch<offer::ZwlrDataControlOfferV1, ()> for Server {
    fn event(
        _: &mut Self,
        _: &offer::ZwlrDataControlOfferV1,
        _: offer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for Server {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<manager::ZwlrDataControlManagerV1, ()> for Server {
    fn event(
        _: &mut Self,
        _: &manager::ZwlrDataControlManagerV1,
        _: manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
