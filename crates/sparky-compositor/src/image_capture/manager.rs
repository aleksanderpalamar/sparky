use std::sync::{Arc, Mutex};
use smithay::reexports::wayland_protocols::ext::image_copy_capture::v1::server::ext_image_copy_capture_manager_v1::{Request, Error, Options};
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource, WEnum};
use crate::state::Sparky;
use super::types::{Manager, Target, OutputManager, WindowManager, SessionData, SessionStatus};

pub fn register(display: &DisplayHandle) {
    display.create_global::<Sparky, Manager, _>(1, ());
    display.create_global::<Sparky, OutputManager, _>(1, ());
    display.create_global::<Sparky, WindowManager, _>(1, ());
}

impl GlobalDispatch<Manager, ()> for Sparky {
    fn bind(
        _: &mut Self,
        _: &DisplayHandle,
        _: &Client,
        resource: New<Manager>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        init.init(resource, ());
    }
}
impl Dispatch<Manager, ()> for Sparky {
    fn request(
        state: &mut Self,
        _: &Client,
        manager: &Manager,
        request: Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        match request {
            Request::CreateSession {
                session,
                source,
                options,
            } => {
                let WEnum::Value(options) = options else {
                    manager.post_error(Error::InvalidOption, "opção de captura inválida");
                    return;
                };
                if options.bits() & !Options::PaintCursors.bits() != 0 {
                    manager.post_error(Error::InvalidOption, "opção de captura inválida");
                    return;
                }
                let target = source
                    .data::<Target>()
                    .cloned()
                    .unwrap_or(Target::Unavailable);
                let size = state.capture_dimensions(&target);
                let data = Arc::new(SessionData {
                    target,
                    cursor: options.contains(Options::PaintCursors),
                    status: Mutex::new(SessionStatus::default()),
                });
                let session = init.init(session, data.clone());
                state
                    .image_capture
                    .sessions
                    .retain(|weak| weak.upgrade().is_ok());
                let Some(size) = size else {
                    data.stop(&session);
                    return;
                };
                if state.lock.is_locked()
                    || state.image_capture.sessions.len() >= super::rules::MAX_SESSIONS
                    || state.image_capture.formats.entries.is_empty()
                {
                    data.stop(&session);
                    return;
                }
                tracing::info!(target = ?data.target, width = size.0, height = size.1, "sessão de captura DMA-BUF criada");
                if let Ok(mut status) = data.status.lock() {
                    status.size = size;
                }
                state.image_capture.formats.announce(&session, size);
                state.image_capture.sessions.push(session.downgrade());
            }
            Request::CreatePointerCursorSession { session, .. } => {
                init.init(session, std::sync::atomic::AtomicBool::new(false));
            }
            _ => {}
        }
    }
}
