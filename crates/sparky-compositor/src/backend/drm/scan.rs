use smithay::backend::drm::DrmDevice;
use smithay::reexports::drm::control::{Device as ControlDevice, ResourceHandles, connector, crtc};

use crate::backend::drm::crtcs::Candidate;
use crate::backend::drm::screens::output_name;

pub type Candidates = (Vec<String>, Vec<Candidate<connector::Handle, crtc::Handle>>);

pub fn candidates(device: &DrmDevice) -> Option<Candidates> {
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
