use smithay::backend::allocator::Buffer;
use smithay::reexports::wayland_protocols::ext::image_copy_capture::v1::server::ext_image_copy_capture_frame_v1::{Request, Error, FailureReason};
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, Resource};
use smithay::wayland::dmabuf::get_dmabuf;
use crate::state::Sparky;
use super::types::{Frame, FrameData};
use super::frame_state::FrameError;
use super::queue::{Job, Notify};

impl Dispatch<Frame, FrameData> for Sparky {
    fn request(
        state: &mut Self,
        _: &Client,
        frame: &Frame,
        request: Request,
        data: &FrameData,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        let Ok(mut pending) = data.state.lock() else {
            frame.failed(FailureReason::Unknown);
            return;
        };
        let outcome = match request {
            Request::AttachBuffer { buffer } => pending.attach(buffer),
            Request::DamageBuffer {
                x,
                y,
                width,
                height,
            } => pending.damage(x, y, width, height),
            Request::Capture => match pending.capture() {
                Ok(buffer) => {
                    drop(pending);
                    state.capture_buffer(frame, data, &buffer);
                    return;
                }
                Err(error) => Err(error),
            },
            _ => return,
        };
        if let Err(error) = outcome {
            frame_error(frame, error);
        }
    }
}

fn frame_error(frame: &Frame, error: FrameError) {
    let (code, message) = match error {
        FrameError::NoBuffer => (Error::NoBuffer, "capture requer um buffer"),
        FrameError::AlreadyCaptured => (Error::AlreadyCaptured, "este quadro já foi capturado"),
        FrameError::InvalidDamage => (Error::InvalidBufferDamage, "retângulo de dano inválido"),
    };
    frame.post_error(code, message);
}

impl Sparky {
    fn capture_buffer(
        &mut self,
        frame: &Frame,
        data: &FrameData,
        buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
        let Ok(status) = data.data.status.lock() else {
            frame.failed(FailureReason::Unknown);
            return;
        };
        if status.stopped {
            frame.failed(FailureReason::Stopped);
            return;
        }
        let size = status.size;
        drop(status);
        let Ok(buffer) = get_dmabuf(buffer) else {
            frame.failed(FailureReason::BufferConstraints);
            return;
        };
        if (buffer.width() as i32, buffer.height() as i32) != size
            || !self
                .image_capture
                .formats
                .entries
                .contains(&buffer.format())
        {
            frame.failed(FailureReason::BufferConstraints);
            return;
        }
        self.queue_capture(Job {
            notify: Notify::Image(frame.clone()),
            buffer: buffer.clone(),
            target: data.data.target.clone(),
            size,
            cursor: data.data.cursor,
            session: Some((data.session.clone(), data.data.clone())),
            region: None,
            locked: false,
        });
    }
}
