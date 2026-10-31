use smithay::backend::drm::DrmDevice;
use smithay::backend::drm::VrrSupport;
use smithay::backend::drm::output::DrmOutputRenderElements;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Mode as WlMode;
use smithay::reexports::drm::control::{Device as ControlDevice, Mode, ModeTypeFlags, connector};
use sparky_config::{OutputMode, OutputRule, rule_for};

use crate::backend::drm::Drm;
use crate::backend::drm::gpu::{Gpu, Surface};
use crate::backend::drm::hdr::{self, Video};
use crate::backend::drm::modes;
use crate::layout::rotation::transform;
use crate::layout::screens::output_mode;
use crate::render::elements::SparkyElement;
use crate::state::Sparky;

impl Sparky {
    pub fn apply_outputs(&mut self) {
        self.with_drm(|state, drm| {
            state.scan_connectors(drm);
            state.adjust_surfaces(drm);
        });
        self.apply_output_scales();
    }

    pub fn output_rule(&self, name: &str) -> OutputRule {
        rule_for(&self.config.output, name)
            .cloned()
            .unwrap_or_else(|| OutputRule::new(name))
    }

    pub fn output_disabled(&self, name: &str) -> bool {
        rule_for(&self.config.output, name).is_some_and(|rule| !rule.enabled)
    }

    fn adjust_surfaces(&mut self, drm: &mut Drm) {
        let Gpu {
            manager,
            renderer,
            surfaces,
            allocator,
            formats,
            ..
        } = &mut drm.gpu;
        for surface in surfaces.values_mut() {
            let rule = self.output_rule(&surface.output.name());
            adjust_mode(manager.device(), renderer, surface, rule.mode);
            surface
                .output
                .change_current_state(None, Some(transform(rule.rotation)), None, None);
            if let Some(on) = rule.vrr {
                set_vrr(surface, on);
            }
            let video = Video {
                device: manager.device(),
                allocator,
                formats,
            };
            hdr::set(&video, surface, rule.hdr);
        }
        self.needs_render = true;
    }
}

pub fn chosen_mode(info: &connector::Info, wanted: Option<OutputMode>) -> Option<Mode> {
    let available: Vec<OutputMode> = info
        .modes()
        .iter()
        .map(|mode| output_mode(WlMode::from(*mode)))
        .collect();
    let index = modes::pick(&available, preferred_index(info), wanted)?;
    info.modes().get(index).copied()
}

pub fn preferred_mode(info: &connector::Info) -> Option<Mode> {
    chosen_mode(info, None)
}

fn preferred_index(info: &connector::Info) -> Option<usize> {
    info.modes()
        .iter()
        .position(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
}

fn adjust_mode(
    device: &DrmDevice,
    renderer: &mut GlesRenderer,
    surface: &mut Surface,
    wanted: Option<OutputMode>,
) {
    let Ok(info) = device.get_connector(surface.connector, false) else {
        return;
    };
    let Some(mode) = chosen_mode(&info, wanted) else {
        return;
    };
    let fresh = WlMode::from(mode);
    if surface.output.current_mode() == Some(fresh) {
        return;
    }
    let elements = DrmOutputRenderElements::<GlesRenderer, SparkyElement<GlesRenderer>>::default();
    match surface.drm_output.use_mode(mode, renderer, &elements) {
        Ok(()) => surface
            .output
            .change_current_state(Some(fresh), None, None, None),
        Err(error) => {
            tracing::warn!(output = %surface.output.name(), "modo de vídeo recusado: {error}")
        }
    }
}

pub fn set_vrr(surface: &Surface, on: bool) {
    let result = surface.drm_output.with_compositor(|compositor| {
        match compositor.vrr_supported(surface.connector) {
            Ok(VrrSupport::NotSupported) | Err(_) => Ok(()),
            Ok(_) => compositor.use_vrr(on).map_err(|error| error.to_string()),
        }
    });
    if let Err(error) = result {
        tracing::warn!(output = %surface.output.name(), "taxa variável recusada: {error}");
    }
}
