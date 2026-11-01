use smithay::backend::drm::DrmDevice;
use smithay::backend::drm::VrrSupport;
use smithay::output::Mode as WlMode;
use smithay::reexports::drm::control::{Device as ControlDevice, connector};
use sparky_ipc::Vrr;

use crate::backend::drm::Drm;
use crate::backend::drm::edid;
use crate::backend::drm::gpu::Surface;
use crate::layout::screens::{Screen, output_mode};

impl Drm {
    pub fn screens(&self) -> Vec<Screen> {
        let device = self.gpu.manager.device();
        let Ok(resources) = device.resource_handles() else {
            return Vec::new();
        };
        resources
            .connectors()
            .iter()
            .filter_map(|handle| {
                let info = device.get_connector(*handle, false).ok()?;
                (info.state() == connector::State::Connected)
                    .then(|| self.screen(device, *handle, &info))
            })
            .collect()
    }

    fn screen(
        &self,
        device: &DrmDevice,
        handle: connector::Handle,
        info: &connector::Info,
    ) -> Screen {
        let surface = self
            .gpu
            .surfaces
            .values()
            .find(|surface| surface.connector == handle);
        Screen {
            name: output_name(info),
            description: description(device, handle),
            modes: info
                .modes()
                .iter()
                .map(|mode| output_mode(WlMode::from(*mode)))
                .collect(),
            vrr: surface.map_or(Vrr::Unsupported, |surface| vrr(surface, handle)),
            enabled: surface.is_some(),
        }
    }
}

pub fn output_name(info: &connector::Info) -> String {
    format!("{}-{}", info.interface().as_str(), info.interface_id())
}

pub fn description(device: &DrmDevice, handle: connector::Handle) -> String {
    edid_blob(device, handle)
        .and_then(|bytes| edid::describe(&bytes))
        .unwrap_or_default()
}

fn edid_blob(device: &DrmDevice, handle: connector::Handle) -> Option<Vec<u8>> {
    let properties = device.get_properties(handle).ok()?;
    let (handles, values) = properties.as_props_and_values();
    let blob = handles.iter().zip(values).find_map(|(property, value)| {
        let info = device.get_property(*property).ok()?;
        (info.name().to_bytes() == b"EDID").then_some(*value)
    })?;
    (blob != 0)
        .then(|| device.get_property_blob(blob).ok())
        .flatten()
}

fn vrr(surface: &Surface, handle: connector::Handle) -> Vrr {
    surface
        .drm_output
        .with_compositor(|compositor| match compositor.vrr_supported(handle) {
            Ok(VrrSupport::NotSupported) | Err(_) => Vrr::Unsupported,
            Ok(_) if compositor.vrr_enabled() => Vrr::On,
            Ok(_) => Vrr::Off,
        })
}
