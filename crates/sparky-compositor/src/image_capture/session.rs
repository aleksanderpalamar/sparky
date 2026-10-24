use std::sync::{Arc, Mutex};
use smithay::reexports::wayland_protocols::ext::image_copy_capture::v1::server::ext_image_copy_capture_session_v1::{Request, Error};
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, Resource};
use crate::state::Sparky;
use super::types::{Session, SessionData, FrameData};
use super::frame_state::FrameState;

impl Dispatch<Session, Arc<SessionData>> for Sparky {
    fn request(
        _: &mut Self,
        _: &Client,
        session: &Session,
        request: Request,
        data: &Arc<SessionData>,
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let Request::CreateFrame { frame } = request {
            let Ok(mut status) = data.status.lock() else {
                session.stopped();
                return;
            };
            if status
                .frame
                .as_ref()
                .and_then(|f| f.upgrade().ok())
                .is_some_and(|f| f.is_alive())
            {
                session.post_error(
                    Error::DuplicateFrame,
                    "destrua o quadro anterior antes de criar outro",
                );
                return;
            }
            let frame = init.init(
                frame,
                FrameData {
                    session: session.clone(),
                    data: data.clone(),
                    state: Mutex::new(FrameState::default()),
                },
            );
            status.frame = Some(frame.downgrade());
        }
    }
}
