use std::collections::HashMap;

use sparky_ipc::GlassArea;

const OWN: &str = "sparky-";
const WHOLE: [&str; 8] = [
    "sparky-bar",
    "sparky-bar-menu",
    "sparky-quick-panel",
    "sparky-notifications",
    "polkit-sparky",
    "sparky-keyring-prompt",
    "sparky-shot-picker",
    "sparky-share-source",
];

#[derive(Debug, Default)]
pub struct Areas {
    told: HashMap<String, GlassArea>,
}

impl Areas {
    pub fn set(&mut self, namespace: String, area: GlassArea) -> bool {
        if !namespace.starts_with(OWN) {
            return false;
        }
        self.told.insert(namespace, area.clone()) != Some(area)
    }

    pub fn of(&self, namespace: &str) -> Option<GlassArea> {
        self.told
            .get(namespace)
            .cloned()
            .or_else(|| WHOLE.contains(&namespace).then_some(GlassArea::Whole))
    }
}
