use smithay::backend::allocator::format::FormatSet;
use smithay::backend::drm::DrmDevice;
use smithay::backend::renderer::gles::GlesTexProgram;
use smithay::reexports::drm::control::{Device as ControlDevice, connector, property};
use sparky_ipc::Hdr;

use crate::backend::drm::gpu::{Allocator, Surface};
use crate::backend::drm::hdr_edid::{HdrCapability, hdr_capability};
use crate::backend::drm::hdr_format::{HDR_FORMATS, SDR_FORMATS, switch_format};
use crate::backend::drm::hdr_metadata::hdr10;
use crate::backend::drm::hdr_props::{self, HdrProperties, Signal};
use crate::backend::drm::hdr_state::{HdrState, Step};
use crate::backend::drm::screens::edid_blob;
use crate::color_management::description::Range;

pub struct Video<'a> {
    pub device: &'a DrmDevice,
    pub allocator: &'a Allocator,
    pub formats: &'a FormatSet,
    pub pq: Option<&'a GlesTexProgram>,
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
    let result = match surface.hdr.step(on) {
        Step::Enable => enable(video, surface),
        Step::Disable { properties, blob } => disable(video, surface, properties, blob),
        Step::Cancel => {
            surface.hdr = HdrState::Off;
            Ok(())
        }
        Step::Keep => Ok(()),
    };
    if let Err(error) = result {
        tracing::warn!(output = %surface.output.name(), "HDR não aplicado: {error}");
    }
}

pub fn start(video: &Video, surface: &mut Surface) {
    if surface.hdr != HdrState::Pending {
        return;
    }
    if let Err(error) = enable(video, surface) {
        if surface.hdr == HdrState::Pending {
            surface.hdr = HdrState::Off;
        }
        tracing::warn!(output = %surface.output.name(), "HDR não aplicado: {error}");
    }
}

pub fn fall_back(video: &Video, surface: &mut Surface, error: &str) -> bool {
    let HdrState::On { .. } = surface.hdr else {
        return false;
    };
    tracing::warn!(output = %surface.output.name(), "HDR desligado nesta sessão, a tela recusou o quadro: {error}");
    set(video, surface, false);
    surface.hdr == HdrState::Off
}

pub fn restore(device: &DrmDevice, surface: &Surface) {
    let HdrState::On {
        properties, blob, ..
    } = surface.hdr
    else {
        return;
    };
    let signal = Signal::hdr(&properties, blob);
    if let Err(error) = hdr_props::send(device, surface.connector, &properties, &signal) {
        tracing::warn!(output = %surface.output.name(), "HDR não restaurado: {error}");
        return;
    }
    resync(surface);
}

pub fn release(device: &DrmDevice, surface: &Surface) {
    if let HdrState::On { blob, .. } = surface.hdr {
        forget(device, blob);
    }
}

fn capability(device: &DrmDevice, handle: connector::Handle) -> HdrCapability {
    edid_blob(device, handle).map_or_else(HdrCapability::default, |bytes| hdr_capability(&bytes))
}

fn signal_on(
    device: &DrmDevice,
    connector: connector::Handle,
) -> Result<(HdrProperties, u64, Range), String> {
    let capability = capability(device, connector);
    if !capability.supported() {
        return Err("o monitor não anuncia HDR10 (PQ e BT.2020)".to_owned());
    }
    let properties =
        hdr_props::find(device, connector).ok_or("o conector não expõe as propriedades de HDR")?;
    let blob = match device.create_property_blob(&hdr10(&capability)) {
        Ok(property::Value::Blob(blob)) => blob,
        Ok(_) => return Err("blob de metadados inválido".to_owned()),
        Err(error) => return Err(error.to_string()),
    };
    let signal = Signal::hdr(&properties, blob);
    if let Err(error) = hdr_props::send(device, connector, &properties, &signal) {
        forget(device, blob);
        return Err(error.to_string());
    }
    Ok((properties, blob, capability.luminance()))
}

fn enable(video: &Video, surface: &mut Surface) -> Result<(), String> {
    if video.pq.is_none() {
        return Err("a conversão para PQ não está disponível".to_owned());
    }
    let (properties, blob, display) = signal_on(video.device, surface.connector)?;
    surface.hdr = HdrState::On {
        properties,
        blob,
        display,
    };
    resync(surface);
    if let Err(error) = switch_format(video, surface, &HDR_FORMATS) {
        disable(video, surface, properties, blob)?;
        return Err(error);
    }
    Ok(())
}

fn disable(
    video: &Video,
    surface: &mut Surface,
    properties: HdrProperties,
    blob: u64,
) -> Result<(), String> {
    let signal = Signal::sdr(&properties);
    hdr_props::send(video.device, surface.connector, &properties, &signal)
        .map_err(|error| error.to_string())?;
    forget(video.device, blob);
    surface.hdr = HdrState::Off;
    surface.composite = None;
    resync(surface);
    if let Err(error) = switch_format(video, surface, &SDR_FORMATS) {
        tracing::warn!(output = %surface.output.name(), "formato SDR mantido: {error}");
    }
    Ok(())
}

fn resync(surface: &Surface) {
    if let Err(error) = surface
        .drm_output
        .with_compositor(|compositor| compositor.reset_state())
    {
        tracing::warn!(output = %surface.output.name(), "estado do vídeo não relido: {error}");
    }
}

fn forget(device: &DrmDevice, blob: u64) {
    if let Err(error) = device.destroy_property_blob(blob) {
        tracing::warn!("metadados HDR não liberados: {error}");
    }
}
