use std::time::Duration;

use smithay::backend::drm::compositor::FrameFlags;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::reexports::drm::control::crtc;
use smithay::utils::Scale;

use crate::backend::drm::Drm;
use crate::backend::drm::gpu::Surface;
use crate::backend::nested::send_frames;
use crate::render::cursor::{Pointer, cursor_elements};
use crate::render::elements::SparkyElement;
use crate::render::output::output_elements;
use crate::state::Sparky;

const IDLE_FRAME: Duration = Duration::from_millis(16);

pub fn render_all(state: &mut Sparky, drm: &mut Drm) {
    let crtcs: Vec<crtc::Handle> = drm.gpu.surfaces.keys().copied().collect();
    for crtc in crtcs {
        render_surface(state, drm, crtc);
    }
}

fn render_surface(state: &mut Sparky, drm: &mut Drm, crtc: crtc::Handle) {
    if !drm.active {
        return;
    }
    let Some(output) = drm
        .gpu
        .surfaces
        .get(&crtc)
        .filter(|surface| !surface.frame_pending)
        .map(|surface| surface.output.clone())
    else {
        return;
    };
    let mut elements = pointer(state, drm, &output);
    elements.extend(output_elements(state, &mut drm.gpu.renderer, &output));
    let clear = state.colors.background;
    let gpu = &mut drm.gpu;
    let Some(surface) = gpu.surfaces.get_mut(&crtc) else {
        return;
    };
    match surface.drm_output.render_frame(&mut gpu.renderer, &elements, clear, FrameFlags::DEFAULT) {
        Ok(result) if !result.is_empty => queue(surface),
        Ok(_) => schedule_idle_frames(state, surface, crtc),
        Err(error) => tracing::warn!("quadro não desenhado: {error}"),
    }
}

fn pointer(state: &Sparky, drm: &mut Drm, output: &Output) -> Vec<SparkyElement<GlesRenderer>> {
    let (Some(geometry), Some(handle)) = (state.space.output_geometry(output), state.seat.get_pointer()) else {
        return Vec::new();
    };
    let location = handle.current_location();
    if !geometry.to_f64().contains(location) {
        return Vec::new();
    }
    let pointer = Pointer {
        status: &state.cursor_status,
        default: &drm.cursor.buffer,
        default_hotspot: drm.cursor.hotspot,
        location: location - geometry.loc.to_f64(),
    };
    let scale = Scale::from(output.current_scale().fractional_scale());
    cursor_elements(&mut drm.gpu.renderer, &pointer, scale)
}

fn queue(surface: &mut Surface) {
    match surface.drm_output.queue_frame(()) {
        Ok(()) => surface.frame_pending = true,
        Err(error) => tracing::warn!("quadro não enfileirado: {error}"),
    }
}

fn schedule_idle_frames(state: &mut Sparky, surface: &mut Surface, crtc: crtc::Handle) {
    if surface.idle_timer.is_some() {
        return;
    }
    let timer = state.loop_handle.insert_source(Timer::from_duration(IDLE_FRAME), move |_, _, state| {
        state.on_idle_frame(crtc);
        TimeoutAction::Drop
    });
    match timer {
        Ok(token) => surface.idle_timer = Some(token),
        Err(error) => tracing::warn!("sem relógio de quadros: {error}"),
    }
}

impl Sparky {
    pub fn on_vblank(&mut self, crtc: crtc::Handle) {
        self.with_drm(|state, drm| {
            let Some(surface) = drm.gpu.surfaces.get_mut(&crtc) else {
                return;
            };
            if let Err(error) = surface.drm_output.frame_submitted() {
                tracing::warn!("quadro não confirmado: {error}");
            }
            surface.frame_pending = false;
            let output = surface.output.clone();
            send_frames(state, &output);
            render_surface(state, drm, crtc);
        });
    }

    fn on_idle_frame(&mut self, crtc: crtc::Handle) {
        self.with_drm(|state, drm| {
            let Some(surface) = drm.gpu.surfaces.get_mut(&crtc) else {
                return;
            };
            surface.idle_timer = None;
            let output = surface.output.clone();
            send_frames(state, &output);
        });
    }
}
