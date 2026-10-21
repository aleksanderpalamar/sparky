pub mod client;
pub mod listener;
pub mod protocols;

use std::ffi::OsString;
use std::time::{Instant, SystemTime};

use smithay::desktop::{PopupManager, Space, Window};
use smithay::input::Seat;
use smithay::input::pointer::CursorImageStatus;
use smithay::wayland::dmabuf::DmabufGlobal;
use smithay::xwayland::X11Wm;
use smithay::reexports::calloop::{EventLoop, LoopHandle, LoopSignal, RegistrationToken};
use smithay::reexports::wayland_server::{Display, DisplayHandle};
use sparky_config::Config;
use sparky_tiling::Desktop;

use crate::backend::Backend;
use crate::bindings::Bindings;
use crate::config::{self, keyboard};
use crate::ipc::server::IpcServer;
use crate::render::colors::BorderColors;
use crate::render::frames::Frames;
use crate::lock::Lock;
use crate::tabs::lettering::Lettering;
use crate::wallpaper::Wallpaper;
use crate::tabs::paint::TabPaint;
use crate::state::protocols::Protocols;
use crate::windows::registry::Registry;

pub struct Sparky {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, Sparky>,
    pub loop_signal: LoopSignal,
    pub socket_name: OsString,
    pub start_time: Instant,
    pub protocols: Protocols,
    pub seat: Seat<Sparky>,
    pub space: Space<Window>,
    pub popups: PopupManager,
    pub desktop: Desktop,
    pub windows: Registry<Window>,
    pub bindings: Bindings,
    pub suppressed_keys: Vec<u32>,
    pub frames: Frames,
    pub tab_paint: TabPaint,
    pub lettering: Lettering,
    pub lock: Lock,
    pub wallpaper: Wallpaper,
    pub colors: BorderColors,
    pub backend: Backend,
    pub cursor_status: CursorImageStatus,
    pub dmabuf_global: Option<DmabufGlobal>,
    pub needs_render: bool,
    pub xwm: Option<X11Wm>,
    pub x_display: Option<u32>,
    pub unmanaged: Vec<Window>,
    pub ipc: Option<IpcServer>,
    pub xwayland_source: Option<RegistrationToken>,
    pub config: Config,
    pub config_stamp: Option<SystemTime>,
}

impl Sparky {
    pub fn new(event_loop: &mut EventLoop<'static, Sparky>, display: Display<Sparky>) -> Result<Self, Box<dyn std::error::Error>> {
        let display_handle = display.handle();
        let mut protocols = Protocols::new(&display_handle);
        let mut seat = protocols.seat.new_wl_seat(&display_handle, "seat0");
        let (config, config_stamp) = config::initial();
        let layout = config::keyboard_for(&config);
        let input = &config.input;
        seat.add_keyboard(keyboard::xkb_config(&layout), input.repeat_delay, input.repeat_rate)?;
        seat.add_pointer();
        let loop_handle = event_loop.handle();
        let socket_name = listener::listen(display, &loop_handle)?;
        let wallpaper = Wallpaper::new(&loop_handle);
        Ok(Self {
            display_handle,
            loop_handle,
            loop_signal: event_loop.get_signal(),
            socket_name,
            start_time: Instant::now(),
            protocols,
            seat,
            space: Space::default(),
            popups: PopupManager::default(),
            desktop: Desktop::new(config.general.style()),
            windows: Registry::default(),
            bindings: config::bindings_for(&config),
            suppressed_keys: Vec::new(),
            frames: Frames::default(),
            tab_paint: TabPaint::default(),
            lettering: Lettering::default(),
            lock: Lock::default(),
            wallpaper,
            colors: BorderColors::from_palette(sparky_theme::current_palette()),
            backend: Backend::Starting,
            cursor_status: CursorImageStatus::default_named(),
            dmabuf_global: None,
            needs_render: true,
            xwm: None,
            x_display: None,
            unmanaged: Vec::new(),
            ipc: None,
            xwayland_source: None,
            config,
            config_stamp,
        })
    }

    pub fn after_dispatch(&mut self) {
        if std::mem::take(&mut self.needs_render) {
            self.render_pending();
        }
        self.broadcast_changes();
        self.space.refresh();
        self.popups.cleanup();
        if let Err(error) = self.display_handle.flush_clients() {
            tracing::warn!("falha ao enviar aos clientes: {error}");
        }
    }
}
