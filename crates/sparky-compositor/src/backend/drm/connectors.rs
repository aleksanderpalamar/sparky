use smithay::backend::drm::DrmDevice;
use smithay::backend::drm::output::DrmOutputRenderElements;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::{Mode as WlMode, Output, PhysicalProperties, Subpixel};
use smithay::reexports::drm::control::{
    Device as ControlDevice, Mode, ResourceHandles, connector, crtc,
};
use sparky_tiling::Rect;

use crate::backend::drm::Drm;
use crate::backend::drm::configure::{chosen_mode, preferred_mode, set_vrr};
use crate::backend::drm::crtcs::{self, Candidate};
use crate::backend::drm::gpu::{Gpu, Surface};
use crate::backend::drm::hdr;
use crate::backend::drm::hdr_state::HdrState;
use crate::backend::drm::modes;
use crate::backend::drm::placement;
use crate::backend::drm::screens::{description, output_name};
use crate::layout::outputs::rect_of;
use crate::layout::rotation::transform;
use crate::render::elements::SparkyElement;
use crate::state::Sparky;

impl Sparky {
    pub fn scan_connectors(&mut self, drm: &mut Drm) {
        let Some((names, found)) = candidates(drm.gpu.manager.device()) else {
            return;
        };
        let kept = modes::kept(&names, |name| self.output_disabled(name));
        let found: Vec<_> = found
            .into_iter()
            .zip(kept)
            .filter(|(_, kept)| *kept)
            .map(|(candidate, _)| candidate)
            .collect();
        let plan = crtcs::plan(&found, &drm.gpu.assigned());
        for connector in plan.disconnect {
            self.disconnect(&mut drm.gpu, connector);
        }
        for (connector, crtc) in plan.connect {
            if let Err(error) = self.connect(&mut drm.gpu, connector, crtc) {
                tracing::warn!("monitor não ligado: {error}");
            }
        }
        self.sync_outputs();
        self.relayout();
        self.needs_render = true;
    }

    fn connect(
        &mut self,
        gpu: &mut Gpu,
        connector: connector::Handle,
        crtc: crtc::Handle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let info = gpu.manager.device().get_connector(connector, false)?;
        let rule = self.output_rule(&output_name(&info));
        let mode = chosen_mode(&info, rule.mode).ok_or("conector sem modo de vídeo")?;
        let label = description(gpu.manager.device(), connector);
        let output = self.create_output(&info, mode, &label);
        let global = output.create_global::<Sparky>(&self.display_handle);
        let drm_output = gpu
            .manager
            .initialize_output::<_, SparkyElement<GlesRenderer>>(
                crtc,
                mode,
                &[connector],
                &output,
                None,
                &mut gpu.renderer,
                &DrmOutputRenderElements::default(),
            );
        let drm_output = match drm_output {
            Ok(drm_output) => drm_output,
            Err(error) => {
                self.space.unmap_output(&output);
                self.display_handle.remove_global::<Sparky>(global);
                return Err(error.into());
            }
        };
        tracing::info!("monitor {} ligado", output.name());
        let surface = Surface {
            output,
            global,
            drm_output,
            connector,
            frame_pending: false,
            idle_timer: None,
            hdr: HdrState::requested(rule.hdr),
            composite: None,
        };
        if let Some(on) = rule.vrr {
            set_vrr(&surface, on);
        }
        gpu.surfaces.insert(crtc, surface);
        Ok(())
    }

    fn create_output(&mut self, info: &connector::Info, mode: Mode, label: &str) -> Output {
        let name = output_name(info);
        let (width, height) = info.size().unwrap_or((0, 0));
        let (make, model) = label.split_once(' ').unwrap_or((label, ""));
        let known = |text: &str| match text.is_empty() {
            true => "Desconhecido".to_owned(),
            false => text.to_owned(),
        };
        let output = Output::new(
            name,
            PhysicalProperties {
                size: (width as i32, height as i32).into(),
                subpixel: Subpixel::Unknown,
                make: known(make),
                model: known(model),
            },
        );
        let existing: Vec<Rect> = self
            .space
            .outputs()
            .filter_map(|known| self.space.output_geometry(known))
            .map(rect_of)
            .collect();
        let position = placement::next_position(&existing);
        let wl_mode = WlMode::from(mode);
        output.set_preferred(preferred_mode(info).map_or(wl_mode, WlMode::from));
        let scale = self.configured_scale(&output.name());
        let rotation = transform(self.output_rule(&output.name()).rotation);
        output.change_current_state(
            Some(wl_mode),
            Some(rotation),
            Some(scale),
            Some(position.into()),
        );
        self.space.map_output(&output, position);
        output
    }

    fn disconnect(&mut self, gpu: &mut Gpu, connector: connector::Handle) {
        let crtc = gpu
            .surfaces
            .iter()
            .find(|(_, surface)| surface.connector == connector)
            .map(|(crtc, _)| *crtc);
        let Some(mut surface) = crtc.and_then(|crtc| gpu.surfaces.remove(&crtc)) else {
            return;
        };
        hdr::set(&gpu.video(), &mut surface, false);
        hdr::release(gpu.manager.device(), &surface);
        tracing::info!("monitor {} desligado", surface.output.name());
        self.space.unmap_output(&surface.output);
        self.display_handle.remove_global::<Sparky>(surface.global);
        if let Some(timer) = surface.idle_timer {
            self.loop_handle.remove(timer);
        }
    }
}

type Candidates = (Vec<String>, Vec<Candidate<connector::Handle, crtc::Handle>>);

fn candidates(device: &DrmDevice) -> Option<Candidates> {
    let resources = device.resource_handles().ok()?;
    let found = resources
        .connectors()
        .iter()
        .filter_map(|handle| {
            let info = device.get_connector(*handle, true).ok()?;
            (info.state() == connector::State::Connected).then(|| {
                let candidate = Candidate {
                    connector: *handle,
                    crtcs: crtcs_for(device, &resources, &info),
                };
                (output_name(&info), candidate)
            })
        })
        .unzip();
    Some(found)
}

fn crtcs_for(
    device: &DrmDevice,
    resources: &ResourceHandles,
    info: &connector::Info,
) -> Vec<crtc::Handle> {
    info.encoders()
        .iter()
        .filter_map(|encoder| device.get_encoder(*encoder).ok())
        .flat_map(|encoder| resources.filter_crtcs(encoder.possible_crtcs()))
        .collect()
}
