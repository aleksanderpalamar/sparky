pub mod fields;
pub mod port;
pub mod socket;
pub mod source;

#[cfg(test)]
mod ipc_tests;
#[cfg(test)]
mod socket_tests;
#[cfg(test)]
mod source_tests;

use std::sync::Arc;

use crate::config::settings::Settings;
use crate::ipc::socket::SocketCompositor;
use crate::ipc::source::SparkySource;
use crate::sources::Source;

pub fn sources(settings: &Settings) -> Vec<Arc<dyn Source>> {
    let compositor = SocketCompositor::from_env(settings.timeout);
    vec![Arc::new(SparkySource::new(compositor))]
}
