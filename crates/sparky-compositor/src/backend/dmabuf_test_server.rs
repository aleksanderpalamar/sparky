use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::backend::allocator::format::FormatSet;
use smithay::backend::allocator::{Format, Fourcc, Modifier};
use smithay::reexports::wayland_server::Display;
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier};

pub type Error = Box<dyn std::error::Error>;

pub const DEVICE: libc::dev_t = 0x1234;

pub fn formats() -> FormatSet {
    [
        Format {
            code: Fourcc::Argb8888,
            modifier: Modifier::Linear,
        },
        Format {
            code: Fourcc::Xrgb8888,
            modifier: Modifier::Invalid,
        },
    ]
    .into_iter()
    .collect()
}

struct State(DmabufState);

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl DmabufHandler for State {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.0
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, _buffer: Dmabuf, notifier: ImportNotifier) {
        notifier.failed();
    }
}

smithay::delegate_dmabuf!(State);

pub struct Server {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<(), String>>>,
}

impl Server {
    pub fn connect(device: Option<libc::dev_t>) -> Result<(Self, wayland_client::Connection), Error> {
        let (server, client) = UnixStream::pair()?;
        let mut display = Display::<State>::new()?;
        let mut state = State(DmabufState::new());
        let mut handle = display.handle();
        let _client = handle.insert_client(server, Arc::new(()))?;
        super::publish::<State>(&handle, &mut state.0, device, formats())?;
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let worker = thread::spawn(move || {
            while !stopped.load(Ordering::Acquire) {
                display
                    .dispatch_clients(&mut state)
                    .map_err(|error| error.to_string())?;
                display.flush_clients().map_err(|error| error.to_string())?;
                thread::sleep(Duration::from_millis(1));
            }
            Ok(())
        });
        let server = Self {
            stop,
            worker: Some(worker),
        };
        let connection = wayland_client::Connection::from_socket(client)?;
        Ok((server, connection))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let Some(worker) = self.worker.take() else {
            return;
        };
        match worker.join() {
            Ok(Ok(())) => {}
            result => eprintln!("servidor de teste DMA-BUF: {result:?}"),
        }
    }
}
