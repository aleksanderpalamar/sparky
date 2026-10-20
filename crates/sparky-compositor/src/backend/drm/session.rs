use smithay::backend::input::InputEvent;
use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
use smithay::backend::session::libseat::{LibSeatSession, LibSeatSessionNotifier};
use smithay::backend::session::Event as SessionEvent;
use smithay::backend::udev::{UdevBackend, UdevEvent};
use smithay::reexports::calloop::LoopHandle;
use smithay::reexports::input::{ClickMethod, Device, Libinput};
use smithay::reexports::rustix::fs::Dev;

use crate::backend::Backend;
use crate::state::Sparky;

pub fn start_input(session: &LibSeatSession, seat: &str, handle: &LoopHandle<'static, Sparky>) -> Result<Libinput, Box<dyn std::error::Error>> {
    let mut libinput = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(session.clone().into());
    libinput
        .udev_assign_seat(seat)
        .map_err(|_| format!("o libinput não assumiu o seat {seat}"))?;
    let backend = LibinputInputBackend::new(libinput.clone());
    handle.insert_source(backend, |mut event, _, state| {
        if let InputEvent::DeviceAdded { device } = &mut event {
            configure(device);
        }
        state.process_input_event(event);
    })?;
    Ok(libinput)
}

pub fn watch_session(notifier: LibSeatSessionNotifier, handle: &LoopHandle<'static, Sparky>) -> Result<(), Box<dyn std::error::Error>> {
    handle.insert_source(notifier, |event, _, state| match event {
        SessionEvent::PauseSession => state.pause_session(),
        SessionEvent::ActivateSession => state.activate_session(),
    })?;
    Ok(())
}

pub fn watch_devices(seat: &str, handle: &LoopHandle<'static, Sparky>) -> Result<(), Box<dyn std::error::Error>> {
    let udev = UdevBackend::new(seat)?;
    handle.insert_source(udev, |event, _, state| {
        if let UdevEvent::Changed { device_id } = event {
            state.on_device_changed(device_id);
        }
    })?;
    Ok(())
}

fn configure(device: &mut Device) {
    if device.config_tap_finger_count() == 0 {
        return;
    }
    if device.config_tap_set_enabled(true).is_err() {
        tracing::warn!("toque para clicar indisponível em {}", device.name());
    }
    if device.config_click_set_method(ClickMethod::Clickfinger).is_err() {
        tracing::warn!("clique por dedos indisponível em {}", device.name());
    }
}

impl Sparky {
    fn pause_session(&mut self) {
        if let Backend::Drm(drm) = &mut self.backend {
            drm.libinput.suspend();
            drm.gpu.manager.pause();
            drm.active = false;
        }
    }

    fn activate_session(&mut self) {
        let Backend::Drm(drm) = &mut self.backend else {
            return;
        };
        if drm.libinput.resume().is_err() {
            tracing::warn!("o libinput não voltou");
        }
        if let Err(error) = drm.gpu.manager.activate(false) {
            tracing::warn!("o vídeo não voltou: {error}");
        }
        for surface in drm.gpu.surfaces.values_mut() {
            surface.frame_pending = false;
        }
        drm.active = true;
        self.needs_render = true;
    }

    fn on_device_changed(&mut self, device: Dev) {
        self.with_drm(|state, drm| {
            if drm.gpu.node.dev_id() == device {
                state.scan_connectors(drm);
            }
        });
    }
}
