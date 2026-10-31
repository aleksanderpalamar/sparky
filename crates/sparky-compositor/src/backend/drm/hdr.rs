use smithay::backend::allocator::Fourcc;
use smithay::backend::allocator::format::FormatSet;
use smithay::backend::drm::DrmDevice;
use smithay::reexports::drm::control::{Device as ControlDevice, connector, property};
use sparky_ipc::Hdr;

use crate::backend::drm::gpu::{Allocator, Surface};
use crate::backend::drm::hdr_edid::{HdrCapability, hdr_capability};
use crate::backend::drm::hdr_metadata::hdr10;
use crate::backend::drm::hdr_props::{self, HdrProperties, Signal};
use crate::backend::drm::screens::edid_blob;

pub const HDR_BPC: u64 = 10;
const HDR_FORMATS: [Fourcc; 2] = [Fourcc::Xrgb2101010, Fourcc::Xbgr2101010];
const SDR_FORMATS: [Fourcc; 2] = [Fourcc::Argb8888, Fourcc::Abgr8888];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HdrState {
    #[default]
    Off,
    On {
        properties: HdrProperties,
        blob: u64,
    },
}

pub struct Video<'a> {
    pub device: &'a DrmDevice,
    pub allocator: &'a Allocator,
    pub formats: &'a FormatSet,
}

pub fn state(device: &DrmDevice, handle: connector::Handle, surface: Option<&Surface>) -> Hdr {
    if !capability(device, handle).supported() || hdr_props::find(device, handle).is_none() {
        return Hdr::Unsupported;
    }
    match surface.map(|surface| surface.hdr) {
        Some(HdrState::On { .. }) => Hdr::On,
        _ => Hdr::Off,
    }
}

pub fn set(video: &Video, surface: &mut Surface, on: bool) {
    let result = match (on, surface.hdr) {
        (true, HdrState::Off) => enable(video, surface),
        (false, HdrState::On { properties, blob }) => disable(video, surface, properties, blob),
        _ => Ok(()),
    };
    if let Err(error) = result {
        tracing::warn!(output = %surface.output.name(), "HDR não aplicado: {error}");
    }
}

pub fn restore(device: &DrmDevice, surface: &Surface) {
    let HdrState::On { properties, blob } = surface.hdr else {
        return;
    };
    let signal = Signal {
        blob,
        colorspace: properties.bt2020,
        bpc: HDR_BPC,
    };
    if let Err(error) = hdr_props::send(device, surface.connector, &properties, &signal) {
        tracing::warn!(output = %surface.output.name(), "HDR não restaurado: {error}");
    }
}

pub fn release(device: &DrmDevice, surface: &Surface) {
    if let HdrState::On { blob, .. } = surface.hdr {
        forget(device, blob);
    }
}

fn capability(device: &DrmDevice, handle: connector::Handle) -> HdrCapability {
    edid_blob(device, handle).map_or_else(HdrCapability::default, |bytes| hdr_capability(&bytes))
}

fn enable(video: &Video, surface: &mut Surface) -> Result<(), String> {
    let capability = capability(video.device, surface.connector);
    if !capability.supported() {
        return Err("o monitor não anuncia HDR10 (PQ e BT.2020)".to_owned());
    }
    let properties = hdr_props::find(video.device, surface.connector)
        .ok_or("o conector não expõe as propriedades de HDR")?;
    let blob = match video.device.create_property_blob(&hdr10(&capability)) {
        Ok(property::Value::Blob(blob)) => blob,
        Ok(_) => return Err("blob de metadados inválido".to_owned()),
        Err(error) => return Err(error.to_string()),
    };
    let signal = Signal {
        blob,
        colorspace: properties.bt2020,
        bpc: HDR_BPC,
    };
    if let Err(error) = hdr_props::send(video.device, surface.connector, &properties, &signal) {
        forget(video.device, blob);
        return Err(error.to_string());
    }
    surface.hdr = HdrState::On { properties, blob };
    switch_format(video, surface, &HDR_FORMATS);
    Ok(())
}

fn disable(
    video: &Video,
    surface: &mut Surface,
    properties: HdrProperties,
    blob: u64,
) -> Result<(), String> {
    let signal = Signal {
        blob: 0,
        colorspace: properties.default_colorspace,
        bpc: properties.current_bpc,
    };
    hdr_props::send(video.device, surface.connector, &properties, &signal)
        .map_err(|error| error.to_string())?;
    forget(video.device, blob);
    surface.hdr = HdrState::Off;
    switch_format(video, surface, &SDR_FORMATS);
    Ok(())
}

fn forget(device: &DrmDevice, blob: u64) {
    if let Err(error) = device.destroy_property_blob(blob) {
        tracing::warn!("metadados HDR não liberados: {error}");
    }
}

fn switch_format(video: &Video, surface: &Surface, formats: &[Fourcc]) {
    let chosen = surface.drm_output.with_compositor(|compositor| {
        formats.iter().copied().find(|code| {
            let modifiers = video
                .formats
                .iter()
                .filter(|format| format.code == *code)
                .map(|format| format.modifier);
            compositor
                .set_format(video.allocator.clone(), *code, modifiers)
                .is_ok()
        })
    });
    match chosen {
        Some(code) => tracing::info!(output = %surface.output.name(), "formato de saída {code:?}"),
        None => tracing::warn!(output = %surface.output.name(), "formato de saída mantido"),
    }
}
