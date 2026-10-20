use std::collections::HashMap;
use std::path::Path;

use smithay::backend::allocator::Fourcc;
use smithay::backend::allocator::gbm::{GbmAllocator, GbmBufferFlags, GbmDevice};
use smithay::backend::drm::exporter::gbm::GbmFramebufferExporter;
use smithay::backend::drm::output::{DrmOutput, DrmOutputManager};
use smithay::backend::drm::{DrmDevice, DrmDeviceFd, DrmEvent, DrmNode};
use smithay::backend::egl::{EGLContext, EGLDisplay};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::session::Session;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::output::Output;
use smithay::reexports::calloop::{LoopHandle, RegistrationToken};
use smithay::reexports::drm::control::{connector, crtc};
use smithay::reexports::rustix::fs::OFlags;
use smithay::reexports::wayland_server::backend::GlobalId;
use smithay::utils::DeviceFd;

use crate::state::Sparky;

const COLOR_FORMATS: [Fourcc; 2] = [Fourcc::Argb8888, Fourcc::Abgr8888];

pub type Allocator = GbmAllocator<DrmDeviceFd>;
pub type Exporter = GbmFramebufferExporter<DrmDeviceFd>;
pub type Manager = DrmOutputManager<Allocator, Exporter, (), DrmDeviceFd>;
pub type SurfaceOutput = DrmOutput<Allocator, Exporter, (), DrmDeviceFd>;

pub struct Surface {
    pub output: Output,
    pub global: GlobalId,
    pub drm_output: SurfaceOutput,
    pub connector: connector::Handle,
    pub frame_pending: bool,
    pub idle_timer: Option<RegistrationToken>,
}

pub struct Gpu {
    pub node: DrmNode,
    pub manager: Manager,
    pub renderer: GlesRenderer,
    pub surfaces: HashMap<crtc::Handle, Surface>,
    pub events: RegistrationToken,
}

impl Gpu {
    pub fn open(
        session: &mut LibSeatSession,
        node: DrmNode,
        path: &Path,
        handle: &LoopHandle<'static, Sparky>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
        let fd = DrmDeviceFd::new(DeviceFd::from(session.open(path, flags)?));
        let (drm, notifier) = DrmDevice::new(fd.clone(), true)?;
        let gbm = GbmDevice::new(fd)?;
        let display = unsafe { EGLDisplay::new(gbm.clone())? };
        let renderer = unsafe { GlesRenderer::new(EGLContext::new(&display)?)? };
        let render_formats = renderer.egl_context().dmabuf_render_formats().clone();
        let allocator = GbmAllocator::new(gbm.clone(), GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT);
        let exporter = GbmFramebufferExporter::new(gbm.clone(), Some(node));
        let manager = DrmOutputManager::new(drm, allocator, exporter, Some(gbm), COLOR_FORMATS, render_formats);
        let events = handle.insert_source(notifier, |event, _, state| match event {
            DrmEvent::VBlank(crtc) => state.on_vblank(crtc),
            DrmEvent::Error(error) => tracing::warn!("erro no dispositivo de vídeo: {error}"),
        })?;
        Ok(Self {
            node,
            manager,
            renderer,
            surfaces: HashMap::new(),
            events,
        })
    }

    pub fn assigned(&self) -> Vec<(connector::Handle, crtc::Handle)> {
        self.surfaces
            .iter()
            .map(|(crtc, surface)| (surface.connector, *crtc))
            .collect()
    }
}
