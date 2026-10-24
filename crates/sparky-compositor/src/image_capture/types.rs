use std::sync::{Arc, Mutex};

use smithay::output::WeakOutput;
use smithay::reexports::wayland_server::{Weak, protocol::wl_buffer::WlBuffer};
use sparky_tiling::WindowId;

use super::frame_state::FrameState;
use super::rules::Pace;

pub use smithay::reexports::wayland_protocols::ext::image_capture_source::v1::server::{
    ext_foreign_toplevel_image_capture_source_manager_v1::ExtForeignToplevelImageCaptureSourceManagerV1 as WindowManager,
    ext_image_capture_source_v1::ExtImageCaptureSourceV1 as Source,
    ext_output_image_capture_source_manager_v1::ExtOutputImageCaptureSourceManagerV1 as OutputManager,
};
pub use smithay::reexports::wayland_protocols::ext::image_copy_capture::v1::server::{
    ext_image_copy_capture_cursor_session_v1::ExtImageCopyCaptureCursorSessionV1 as CursorSession,
    ext_image_copy_capture_frame_v1::ExtImageCopyCaptureFrameV1 as Frame,
    ext_image_copy_capture_manager_v1::ExtImageCopyCaptureManagerV1 as Manager,
    ext_image_copy_capture_session_v1::ExtImageCopyCaptureSessionV1 as Session,
};

#[derive(Debug, Clone)]
pub enum Target {
    Output(WeakOutput),
    Window(WindowId),
    Unavailable,
}

pub struct SessionData {
    pub target: Target,
    pub cursor: bool,
    pub status: Mutex<SessionStatus>,
}

impl SessionData {
    pub fn stop(&self, session: &Session) {
        if let Ok(mut status) = self.status.lock() {
            status.stopped = true;
        }
        session.stopped();
    }
}

#[derive(Default)]
pub struct SessionStatus {
    pub size: (i32, i32),
    pub frame: Option<Weak<Frame>>,
    pub pace: Pace,
    pub stopped: bool,
}

pub struct FrameData {
    pub session: Session,
    pub data: Arc<SessionData>,
    pub state: Mutex<FrameState<WlBuffer>>,
}
