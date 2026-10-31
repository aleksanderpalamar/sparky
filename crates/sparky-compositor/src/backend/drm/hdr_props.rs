use std::io;

use smithay::backend::drm::DrmDevice;
use smithay::reexports::drm::control::atomic::AtomicModeReq;
use smithay::reexports::drm::control::{
    AtomicCommitFlags, Device as ControlDevice, connector, property,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HdrProperties {
    pub colorspace: property::Handle,
    pub bt2020: u64,
    pub default_colorspace: u64,
    pub max_bpc: property::Handle,
    pub metadata: property::Handle,
    pub current_bpc: u64,
}

pub struct Signal {
    pub blob: u64,
    pub colorspace: u64,
    pub bpc: u64,
}

pub fn colorspaces(entries: &[(String, u64)]) -> Option<(u64, u64)> {
    let value = |name: &str| {
        entries
            .iter()
            .find(|(entry, _)| entry == name)
            .map(|(_, value)| *value)
    };
    Some((value("BT2020_RGB")?, value("Default")?))
}

pub fn find(device: &DrmDevice, handle: connector::Handle) -> Option<HdrProperties> {
    let set = device.get_properties(handle).ok()?;
    let (handles, values) = set.as_props_and_values();
    let (mut colorspace, mut max_bpc, mut metadata) = (None, None, None);
    for (property, value) in handles.iter().zip(values) {
        let Ok(info) = device.get_property(*property) else {
            continue;
        };
        match info.name().to_bytes() {
            b"Colorspace" => {
                colorspace = enum_entries(&info)
                    .and_then(|entries| colorspaces(&entries))
                    .map(|pair| (*property, pair))
            }
            b"max bpc" => max_bpc = Some((*property, *value)),
            b"HDR_OUTPUT_METADATA" => metadata = Some(*property),
            _ => {}
        }
    }
    let (colorspace, (bt2020, default_colorspace)) = colorspace?;
    let (max_bpc, current_bpc) = max_bpc?;
    Some(HdrProperties {
        colorspace,
        bt2020,
        default_colorspace,
        max_bpc,
        metadata: metadata?,
        current_bpc,
    })
}

pub fn send(
    device: &DrmDevice,
    handle: connector::Handle,
    properties: &HdrProperties,
    signal: &Signal,
) -> io::Result<()> {
    let mut request = AtomicModeReq::new();
    request.add_property(
        handle,
        properties.metadata,
        property::Value::Blob(signal.blob),
    );
    request.add_property(
        handle,
        properties.colorspace,
        property::Value::UnsignedRange(signal.colorspace),
    );
    request.add_property(
        handle,
        properties.max_bpc,
        property::Value::UnsignedRange(signal.bpc),
    );
    device.atomic_commit(AtomicCommitFlags::ALLOW_MODESET, request)
}

fn enum_entries(info: &property::Info) -> Option<Vec<(String, u64)>> {
    let property::ValueType::Enum(values) = info.value_type() else {
        return None;
    };
    let (_, entries) = values.values();
    Some(
        entries
            .iter()
            .map(|entry| (entry.name().to_string_lossy().into_owned(), entry.value()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::colorspaces;

    fn lista(nomes: &[(&str, u64)]) -> Vec<(String, u64)> {
        nomes
            .iter()
            .map(|(nome, valor)| ((*nome).to_owned(), *valor))
            .collect()
    }

    #[test]
    fn acha_bt2020_e_padrao_pelo_nome_do_enum() {
        let amdgpu = lista(&[
            ("Default", 0),
            ("BT709_YCC", 2),
            ("opRGB", 7),
            ("BT2020_RGB", 9),
            ("BT2020_YCC", 10),
        ]);
        assert_eq!(colorspaces(&amdgpu), Some((9, 0)));
        assert_eq!(
            colorspaces(&lista(&[("Default", 0), ("BT2020_YCC", 10)])),
            None
        );
    }
}
