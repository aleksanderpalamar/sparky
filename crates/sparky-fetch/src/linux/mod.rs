use std::sync::Arc;

use crate::config::settings::Settings;
use crate::sources::Source;

pub fn sources(_settings: &Settings) -> Vec<Arc<dyn Source>> {
    Vec::new()
}
