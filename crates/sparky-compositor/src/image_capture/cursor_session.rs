use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use smithay::reexports::wayland_protocols::ext::image_copy_capture::v1::server::ext_image_copy_capture_cursor_session_v1::{Request, Error};
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, Resource};
use crate::state::Sparky;
use super::types::{CursorSession, SessionData, SessionStatus, Target};

impl Dispatch<CursorSession, AtomicBool> for Sparky {
    fn request(
        _: &mut Self,
        _: &Client,
        resource: &CursorSession,
        request: Request,
        used: &AtomicBool,
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let Request::GetCaptureSession { session } = request {
            if used.swap(true, Ordering::Relaxed) {
                resource.post_error(Error::DuplicateSession, "sessão de cursor já criada");
                return;
            }
            let data = Arc::new(SessionData {
                target: Target::Unavailable,
                cursor: false,
                status: Mutex::new(SessionStatus {
                    stopped: true,
                    ..SessionStatus::default()
                }),
            });
            init.init(session, data).stopped();
        }
    }
}
