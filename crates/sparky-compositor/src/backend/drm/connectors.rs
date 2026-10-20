use smithay::backend::drm::DrmDevice;
use smithay::backend::drm::output::DrmOutputRenderElements;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::{Mode as WlMode, Output, PhysicalProperties, Subpixel};
use smithay::reexports::drm::control::{Device as ControlDevice, Mode, ModeTypeFlags, ResourceHandles, connector, crtc};
use sparky_tiling::Rect;

use crate::backend::drm::Drm;
use crate::backend::drm::crtcs::{self, Candidate};
use crate::backend::drm::gpu::{Gpu, Surface};
use crate::backend::drm::placement;
use crate::layout::outputs::rect_of;
use crate::render::elements::SparkyElement;
use crate::state::Sparky;

impl Sparky {
    pub fn scan_connectors(&mut self, drm: &mut Drm) {
        let Some(found) = candidates(drm.gpu.manager.device()) else {
            return;
        };
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

    fn connect(&mut self, gpu: &mut Gpu, connector: connector::Handle, crtc: crtc::Handle) -> Result<(), Box<dyn std::error::Error>> {
        let info = gpu.manager.device().get_connector(connector, false)?;
        let mode = preferred_mode(&info).ok_or("conector sem modo de vídeo")?;
        let output = self.create_output(&info, mode);
        let global = output.create_global::<Sparky>(&self.display_handle);
        let drm_output = gpu.manager.initialize_output::<_, SparkyElement<GlesRenderer>>(
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
        gpu.surfaces.insert(
            crtc,
            Surface {
                output,
                global,
                drm_output,
                connector,
                frame_pending: false,
                idle_timer: None,
            },
        );
        Ok(())
    }

    fn create_output(&mut self, info: &connector::Info, mode: Mode) -> Output {
        let name = format!("{}-{}", info.interface().as_str(), info.interface_id());
        let (width, height) = info.size().unwrap_or((0, 0));
        let output = Output::new(
            name,
            PhysicalProperties {
                size: (width as i32, height as i32).into(),
                subpixel: Subpixel::Unknown,
                make: "Desconhecido".into(),
                model: "Desconhecido".into(),
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
        output.set_preferred(wl_mode);
        output.change_current_state(Some(wl_mode), None, None, Some(position.into()));
        self.space.map_output(&output, position);
        output
    }

    fn disconnect(&mut self, gpu: &mut Gpu, connector: connector::Handle) {
        let crtc = gpu
            .surfaces
            .iter()
            .find(|(_, surface)| surface.connector == connector)
            .map(|(crtc, _)| *crtc);
        let Some(surface) = crtc.and_then(|crtc| gpu.surfaces.remove(&crtc)) else {
            return;
        };
        tracing::info!("monitor {} desligado", surface.output.name());
        self.space.unmap_output(&surface.output);
        self.display_handle.remove_global::<Sparky>(surface.global);
        if let Some(timer) = surface.idle_timer {
            self.loop_handle.remove(timer);
        }
    }
}

fn candidates(device: &DrmDevice) -> Option<Vec<Candidate<connector::Handle, crtc::Handle>>> {
    let resources = device.resource_handles().ok()?;
    let found = resources
        .connectors()
        .iter()
        .filter_map(|handle| {
            let info = device.get_connector(*handle, true).ok()?;
            (info.state() == connector::State::Connected).then(|| Candidate {
                connector: *handle,
                crtcs: crtcs_for(device, &resources, &info),
            })
        })
        .collect();
    Some(found)
}

fn crtcs_for(device: &DrmDevice, resources: &ResourceHandles, info: &connector::Info) -> Vec<crtc::Handle> {
    info.encoders()
        .iter()
        .filter_map(|encoder| device.get_encoder(*encoder).ok())
        .flat_map(|encoder| resources.filter_crtcs(encoder.possible_crtcs()))
        .collect()
}

fn preferred_mode(info: &connector::Info) -> Option<Mode> {
    info.modes()
        .iter()
        .find(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
        .or_else(|| info.modes().first())
        .copied()
}
