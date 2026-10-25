use std::collections::BTreeMap;

use super::types::Session;
use smithay::backend::allocator::{Format, Fourcc};

#[derive(Default)]
pub struct Formats {
    pub device: Option<libc::dev_t>,
    pub entries: Vec<Format>,
}

impl Formats {
    pub fn legacy(&self) -> Option<Fourcc> {
        [Fourcc::Argb8888, Fourcc::Xrgb8888]
            .into_iter()
            .find(|code| self.entries.iter().any(|format| format.code == *code))
    }

    pub fn set(&mut self, device: Option<libc::dev_t>, formats: impl IntoIterator<Item = Format>) {
        self.device = device;
        self.entries = formats
            .into_iter()
            .filter(|f| {
                matches!(
                    f.code,
                    Fourcc::Argb8888 | Fourcc::Xrgb8888 | Fourcc::Abgr8888 | Fourcc::Xbgr8888
                )
            })
            .collect();
    }

    pub fn announce(&self, session: &Session, size: (i32, i32)) {
        session.buffer_size(size.0 as u32, size.1 as u32);
        if let Some(device) = self.device {
            session.dmabuf_device(device.to_ne_bytes().to_vec());
        }
        let mut groups: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
        for format in &self.entries {
            groups
                .entry(format.code as u32)
                .or_default()
                .extend(u64::from(format.modifier).to_ne_bytes());
        }
        for (code, modifiers) in groups {
            session.dmabuf_format(code, modifiers);
        }
        session.done();
    }
}
