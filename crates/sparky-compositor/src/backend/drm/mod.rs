mod connectors;
pub mod crtcs;
pub mod cursor;
pub mod gpu;
pub mod placement;
pub mod render;
pub mod session;
pub mod vt;

use std::path::PathBuf;

use smithay::backend::drm::DrmNode;
use smithay::backend::session::Session;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::udev::{all_gpus, primary_gpu};
use smithay::reexports::input::Libinput;

use crate::backend::Backend;
use crate::backend::drm::cursor::CursorImage;
use crate::backend::drm::gpu::Gpu;
use crate::cursor::CursorPixels;
use crate::state::Sparky;

pub struct Drm {
    pub session: LibSeatSession,
    pub libinput: Libinput,
    pub gpu: Gpu,
    pub cursor: CursorImage,
    pub active: bool,
}

impl Drm {
    pub fn switch_terminal(&mut self, terminal: i32) {
        if let Err(error) = self.session.change_vt(terminal) {
            tracing::warn!("não deu para ir ao terminal {terminal}: {error}");
        }
    }
}

pub fn init(state: &mut Sparky) -> Result<(), Box<dyn std::error::Error>> {
    let (mut session, notifier) = LibSeatSession::new()?;
    let seat = session.seat();
    let (node, path) = find_gpu(&seat)?;
    tracing::info!("placa de vídeo: {}", path.display());
    let gpu = Gpu::open(&mut session, node, &path, &state.loop_handle)?;
    state.publish_formats(&gpu.renderer);
    let libinput = session::start_input(&session, &seat, &state.loop_handle)?;
    session::watch_session(notifier, &state.loop_handle)?;
    session::watch_devices(&seat, &state.loop_handle)?;
    state.backend = Backend::Drm(Box::new(Drm {
        session,
        libinput,
        gpu,
        cursor: CursorImage::from_pixels(&CursorPixels::load()),
        active: true,
    }));
    state.with_drm(|state, drm| state.scan_connectors(drm));
    Ok(())
}

fn find_gpu(seat: &str) -> Result<(DrmNode, PathBuf), Box<dyn std::error::Error>> {
    let path = match primary_gpu(seat)? {
        Some(path) => path,
        None => all_gpus(seat)?.into_iter().next().ok_or("nenhuma placa de vídeo encontrada")?,
    };
    let node = DrmNode::from_path(&path)?;
    Ok((node, path))
}
