use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, QueueHandle};
pub use wayland_protocols::ext::image_capture_source::v1::client::{
    ext_foreign_toplevel_image_capture_source_manager_v1 as window_manager,
    ext_image_capture_source_v1 as source,
    ext_output_image_capture_source_manager_v1 as output_manager,
};
pub use wayland_protocols::ext::image_copy_capture::v1::client::{
    ext_image_copy_capture_frame_v1 as frame, ext_image_copy_capture_manager_v1 as manager,
    ext_image_copy_capture_session_v1 as session,
};

#[derive(Default)]
pub struct Events {
    pub legacy_dma: Option<(u32, u32, u32)>,
    pub legacy_shm: Option<(u32, u32, u32)>,
    pub legacy_done: bool,
    pub windows: Vec<super::window_events::handle::ExtForeignToplevelHandleV1>,
    pub titles: Vec<String>,
    pub closed: usize,
    pub size: Option<(u32, u32)>,
    pub done: usize,
    pub stopped: usize,
    pub formats: Vec<(u32, Vec<u8>)>,
    pub device: Option<Vec<u8>>,
    pub failures: Vec<wayland_client::WEnum<frame::FailureReason>>,
    pub ready: usize,
}

impl Dispatch<session::ExtImageCopyCaptureSessionV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &session::ExtImageCopyCaptureSessionV1,
        event: session::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            session::Event::BufferSize { width, height } => state.size = Some((width, height)),
            session::Event::DmabufFormat { format, modifiers } => {
                state.formats.push((format, modifiers))
            }
            session::Event::DmabufDevice { device } => state.device = Some(device),
            session::Event::Done => state.done += 1,
            session::Event::Stopped => state.stopped += 1,
            _ => {}
        }
    }
}

impl Dispatch<frame::ExtImageCopyCaptureFrameV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &frame::ExtImageCopyCaptureFrameV1,
        event: frame::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            frame::Event::Failed { reason } => state.failures.push(reason),
            frame::Event::Ready => state.ready += 1,
            _ => {}
        }
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
wayland_client::delegate_noop!(Events: ignore wl_output::WlOutput);
wayland_client::delegate_noop!(Events: ignore wl_buffer::WlBuffer);
wayland_client::delegate_noop!(Events: ignore wl_shm::WlShm);
wayland_client::delegate_noop!(Events: ignore wl_shm_pool::WlShmPool);
wayland_client::delegate_noop!(Events: ignore source::ExtImageCaptureSourceV1);
wayland_client::delegate_noop!(Events: ignore output_manager::ExtOutputImageCaptureSourceManagerV1);
wayland_client::delegate_noop!(Events: ignore window_manager::ExtForeignToplevelImageCaptureSourceManagerV1);
wayland_client::delegate_noop!(Events: ignore manager::ExtImageCopyCaptureManagerV1);
