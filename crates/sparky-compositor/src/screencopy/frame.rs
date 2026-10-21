use std::sync::atomic::Ordering;
use std::time::Duration;

use smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_frame_v1::{
    Error, Flags, Request, ZwlrScreencopyFrameV1,
};
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, Resource};
use smithay::utils::{Clock, Monotonic};
use smithay::wayland::shm::with_buffer_contents_mut;

use crate::screencopy::rules::{Shape, accepts, copy_bgra, orient};
use crate::screencopy::{Frame, FrameData};
use crate::state::Sparky;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Damage {
    Reported,
    Skipped,
}

impl Dispatch<ZwlrScreencopyFrameV1, FrameData> for Sparky {
    fn request(
        state: &mut Self,
        _client: &Client,
        frame: &ZwlrScreencopyFrameV1,
        request: Request,
        data: &FrameData,
        _handle: &DisplayHandle,
        _init: &mut DataInit<'_, Self>,
    ) {
        let (buffer, damage) = match request {
            Request::Copy { buffer } => (buffer, Damage::Skipped),
            Request::CopyWithDamage { buffer } => (buffer, Damage::Reported),
            _ => return,
        };
        let FrameData::Ready(spec) = data else {
            frame.failed();
            return;
        };
        if spec.used.swap(true, Ordering::SeqCst) {
            frame.post_error(Error::AlreadyUsed, "este quadro já foi copiado");
            return;
        }
        match state.fill(spec, &buffer) {
            true => ready(frame, spec, damage),
            false => frame.failed(),
        }
    }
}

impl Sparky {
    fn fill(&mut self, spec: &Frame, buffer: &WlBuffer) -> bool {
        let Ok(image) = self.read_region(&spec.output, spec.region) else {
            return false;
        };
        let Some(pixels) = orient(&image.rgba, image.width as usize, spec.output.current_transform()) else {
            return false;
        };
        let written = with_buffer_contents_mut(buffer, |pointer, length, info| {
            let shape = Shape { width: info.width, height: info.height, stride: info.stride, format: info.format };
            let offset = info.offset.max(0) as usize;
            if !accepts(shape, spec.size) || offset > length {
                return false;
            }
            let pool = unsafe { std::slice::from_raw_parts_mut(pointer, length) };
            copy_bgra(&pixels, image.width as usize, &mut pool[offset..], info.stride as usize)
        });
        written.unwrap_or(false)
    }
}

fn ready(frame: &ZwlrScreencopyFrameV1, spec: &Frame, damage: Damage) {
    frame.flags(Flags::empty());
    if damage == Damage::Reported {
        frame.damage(0, 0, spec.size.0 as u32, spec.size.1 as u32);
    }
    let now: Duration = Clock::<Monotonic>::new().now().into();
    let seconds = now.as_secs();
    frame.ready((seconds >> 32) as u32, seconds as u32, now.subsec_nanos());
}
