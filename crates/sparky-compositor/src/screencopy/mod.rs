mod frame;
pub mod rules;

use std::sync::atomic::AtomicBool;

use smithay::output::Output;
use smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1;
use smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_manager_v1::{
    Request, ZwlrScreencopyManagerV1,
};
use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::reexports::wayland_server::protocol::wl_shm::Format;
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource};
use sparky_tiling::Rect;

use crate::capture::render::scaled;
use crate::state::Sparky;

const VERSION: u32 = 3;
const BYTES_PER_PIXEL: i32 = 4;

pub struct ScreencopyState;

impl ScreencopyState {
    pub fn new<D>(display: &DisplayHandle) -> Self
    where
        D: GlobalDispatch<ZwlrScreencopyManagerV1, ()>
            + Dispatch<ZwlrScreencopyManagerV1, ()>
            + 'static,
    {
        display.create_global::<D, ZwlrScreencopyManagerV1, ()>(VERSION, ());
        Self
    }
}

pub struct Frame {
    output: Output,
    region: Rect,
    size: (i32, i32),
    cursor: bool,
    used: AtomicBool,
}

pub enum FrameData {
    Ready(Frame),
    Failed,
}

impl GlobalDispatch<ZwlrScreencopyManagerV1, ()> for Sparky {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<ZwlrScreencopyManagerV1>,
        _data: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        init.init(resource, ());
    }
}

impl Dispatch<ZwlrScreencopyManagerV1, ()> for Sparky {
    fn request(
        state: &mut Self,
        _client: &Client,
        _manager: &ZwlrScreencopyManagerV1,
        request: Request,
        _data: &(),
        _handle: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        match request {
            Request::CaptureOutput {
                frame,
                output,
                overlay_cursor,
            } => state.start_frame(frame, &output, None, overlay_cursor != 0, init),
            Request::CaptureOutputRegion {
                frame,
                output,
                x,
                y,
                width,
                height,
                overlay_cursor,
            } => {
                state.start_frame(
                    frame,
                    &output,
                    Some(Rect::new(x, y, width, height)),
                    overlay_cursor != 0,
                    init,
                );
            }
            _ => {}
        }
    }
}

impl Sparky {
    fn start_frame(
        &self,
        frame: New<ZwlrScreencopyFrameV1>,
        wl_output: &WlOutput,
        requested: Option<Rect>,
        cursor: bool,
        init: &mut DataInit<'_, Self>,
    ) {
        let Some(mut spec) =
            Output::from_resource(wl_output).and_then(|output| self.frame_for(output, requested))
        else {
            init.init(frame, FrameData::Failed).failed();
            return;
        };
        spec.cursor = cursor;
        let (width, height) = spec.size;
        let frame = init.init(frame, FrameData::Ready(spec));
        frame.buffer(
            Format::Argb8888,
            width as u32,
            height as u32,
            (width * BYTES_PER_PIXEL) as u32,
        );
        if frame.version() < VERSION {
            return;
        }
        if let Some(format) = self.image_capture.formats.legacy() {
            frame.linux_dmabuf(format as u32, width as u32, height as u32);
        }
        frame.buffer_done();
    }

    fn frame_for(&self, output: Output, requested: Option<Rect>) -> Option<Frame> {
        let geometry = self.space.output_geometry(&output)?;
        let region = rules::region((geometry.size.w, geometry.size.h), requested)?;
        let physical = scaled(region, output.current_scale().fractional_scale());
        Some(Frame {
            output,
            region,
            size: (physical.width, physical.height),
            cursor: false,
            used: AtomicBool::new(false),
        })
    }
}
