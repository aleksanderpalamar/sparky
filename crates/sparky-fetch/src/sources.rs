use std::sync::Arc;

use crate::config::settings::Settings;
use crate::domain::field::Field;
use crate::domain::status::Status;

pub trait Source: Send + Sync {
    fn fields(&self) -> &'static [Field];
    fn collect(&self, wanted: &[Field]) -> Vec<(Field, Status)>;
}

pub fn all(settings: &Settings) -> Vec<Arc<dyn Source>> {
    let mut sources = crate::linux::sources(settings);
    sources.extend(crate::ipc::sources(settings));
    sources
}
