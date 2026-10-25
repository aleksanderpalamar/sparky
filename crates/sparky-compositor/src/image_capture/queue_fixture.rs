use super::frame_state::FrameState;
use super::queue::{Job, Notify};
use super::types::{FrameData, SessionData, SessionStatus, Target};
use crate::state::{Sparky, client::ClientState};
use smithay::backend::allocator::{
    Fourcc, Modifier,
    dmabuf::{Dmabuf, DmabufFlags},
};
use smithay::output::{Output, PhysicalProperties, Subpixel};
use smithay::reexports::wayland_server::Display;
use std::io::Read;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

pub struct Fixture {
    pub display: Display<Sparky>,
    pub socket: UnixStream,
    pub output: Output,
    pub job: Job,
}

impl Fixture {
    pub fn new(legacy: bool) -> Self {
        let display = Display::<Sparky>::new().unwrap();
        let mut handle = display.handle();
        let (socket, server) = UnixStream::pair().unwrap();
        socket.set_nonblocking(true).unwrap();
        let client = handle
            .insert_client(server, Arc::new(ClientState::default()))
            .unwrap();
        let output = Output::new(
            "fixture".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "fixture".into(),
                model: "fixture".into(),
            },
        );
        let target = Target::Output(output.downgrade());
        let data = Arc::new(SessionData {
            target: target.clone(),
            cursor: false,
            status: Mutex::new(SessionStatus::default()),
        });
        let session = client
            .create_resource::<_, _, Sparky>(&handle, 1, data.clone())
            .unwrap();
        let frame = client
            .create_resource::<_, _, Sparky>(
                &handle,
                1,
                FrameData {
                    session: session.clone(),
                    data: data.clone(),
                    state: Mutex::new(FrameState::default()),
                },
            )
            .unwrap();
        let notify = match legacy {
            true => Notify::Legacy(
                client
                    .create_resource::<_, _, Sparky>(
                        &handle,
                        3,
                        crate::screencopy::FrameData::Failed,
                    )
                    .unwrap(),
                true,
            ),
            false => Notify::Image(frame),
        };
        let mut buffer = Dmabuf::builder(
            (800, 600),
            Fourcc::Argb8888,
            Modifier::Linear,
            DmabufFlags::empty(),
        );
        assert!(buffer.add_plane(tempfile::tempfile().unwrap().into(), 0, 0, 3200));
        Self {
            display,
            socket,
            output,
            job: Job {
                notify,
                buffer: buffer.build().unwrap(),
                target,
                size: (800, 600),
                cursor: false,
                session: (!legacy).then_some((session, data)),
                region: None,
                locked: false,
            },
        }
    }
}

pub fn events(display: &mut Display<Sparky>, socket: &mut UnixStream) -> Vec<u16> {
    display.flush_clients().unwrap();
    let mut bytes = [0; 1024];
    let count = match socket.read(&mut bytes) {
        Ok(count) => count,
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => 0,
        Err(error) => panic!("{error}"),
    };
    let mut offset = 0;
    let mut events = Vec::new();
    while offset < count {
        let header = u32::from_ne_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
        events.push(header as u16);
        offset += (header >> 16) as usize;
    }
    events
}
