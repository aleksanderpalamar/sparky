use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ConnectionExt, EventMask, PropMode, Window,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use super::active::{ActiveWindow, NO_WINDOW, correction};

const ATOM: &[u8] = b"_NET_ACTIVE_WINDOW";

pub struct X11ActiveWindow {
    shared: Arc<Shared>,
}

struct Shared {
    conn: RustConnection,
    root: Window,
    atom: u32,
    wanted: AtomicU32,
}

impl X11ActiveWindow {
    pub fn connect(display: u32) -> Result<Self, Box<dyn Error>> {
        let (conn, screen) = x11rb::connect(Some(&format!(":{display}")))?;
        let root = conn
            .setup()
            .roots
            .get(screen)
            .ok_or("tela X11 inexistente")?
            .root;
        let atom = conn.intern_atom(false, ATOM)?.reply()?.atom;
        let watch = ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE);
        conn.change_window_attributes(root, &watch)?;
        conn.flush()?;
        let shared = Arc::new(Shared {
            conn,
            root,
            atom,
            wanted: AtomicU32::new(NO_WINDOW),
        });
        let watcher = Arc::clone(&shared);
        thread::Builder::new()
            .name("sparky-x11-active".into())
            .spawn(move || watcher.watch())?;
        Ok(Self { shared })
    }
}

impl ActiveWindow for X11ActiveWindow {
    fn announce(&self, window: u32) {
        if self.shared.wanted.swap(window, Ordering::SeqCst) != window {
            self.shared.write(window);
        }
    }
}

impl Shared {
    fn watch(&self) {
        loop {
            match self.conn.wait_for_event() {
                Ok(Event::PropertyNotify(event))
                    if event.window == self.root && event.atom == self.atom =>
                {
                    self.correct();
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::info!("vigia da janela ativa do X11 encerrado: {error}");
                    return;
                }
            }
        }
    }

    fn correct(&self) {
        let wanted = self.wanted.load(Ordering::SeqCst);
        if let Some(window) = correction(self.current(), wanted) {
            self.write(window);
        }
    }

    fn current(&self) -> Option<u32> {
        let cookie = self
            .conn
            .get_property(false, self.root, self.atom, AtomEnum::WINDOW, 0, 1)
            .ok()?;
        let reply = cookie.reply().ok()?;
        reply.value32()?.next()
    }

    fn write(&self, window: u32) {
        let written = self
            .conn
            .change_property32(
                PropMode::REPLACE,
                self.root,
                self.atom,
                AtomEnum::WINDOW,
                &[window],
            )
            .and_then(|_| self.conn.flush());
        if let Err(error) = written {
            tracing::warn!("janela ativa do X11 não anunciada: {error}");
        }
    }
}
