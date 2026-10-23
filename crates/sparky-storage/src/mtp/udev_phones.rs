use std::collections::BTreeSet;

use super::Phones;
use super::identity::phone;
use crate::Volume;

pub struct UdevPhones;

impl Phones for UdevPhones {
    fn connected(&self) -> Result<Vec<Volume>, String> {
        let mut enumerator = udev::Enumerator::new().map_err(failure)?;
        enumerator.match_subsystem("usb").map_err(failure)?;
        enumerator
            .match_property("ID_MTP_DEVICE", "1")
            .map_err(failure)?;
        let mut seen = BTreeSet::new();
        Ok(enumerator
            .scan_devices()
            .map_err(failure)?
            .filter(|device| device.devnode().is_some())
            .filter_map(|device| {
                phone(
                    device
                        .property_value("ID_SERIAL")
                        .and_then(|value| value.to_str()),
                    device
                        .attribute_value("busnum")
                        .and_then(|value| value.to_str()),
                    device
                        .attribute_value("devnum")
                        .and_then(|value| value.to_str()),
                    device
                        .attribute_value("product")
                        .and_then(|value| value.to_str()),
                )
            })
            .filter(|volume| seen.insert(volume.path.clone()))
            .collect())
    }
}

fn failure(error: std::io::Error) -> String {
    format!("Não foi possível consultar os dispositivos MTP: {error}")
}
