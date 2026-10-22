pub mod agent;
pub mod caller;
pub mod daemon;
mod display;
pub mod process;
pub mod registration;
pub mod shared;
pub mod wire;

#[cfg(test)]
mod process_tests;
#[cfg(test)]
mod registration_tests;
#[cfg(test)]
mod service_tests;
#[cfg(test)]
mod test_support;

use futures_util::future::BoxFuture;
use tokio::sync::watch;

use crate::domain::{Outcome, Request};

pub const AUTHORITY: &str = "org.freedesktop.PolicyKit1";
pub const AUTHORITY_PATH: &str = "/org/freedesktop/PolicyKit1/Authority";
pub const PATH: &str = "/org/sparky/PolicyKitAgent";
pub const INTERFACE: &str = "org.freedesktop.PolicyKit1.AuthenticationAgent";

pub trait Runner: Send + Sync {
    fn run(
        &self,
        request: Request,
        cancellation: watch::Receiver<bool>,
    ) -> BoxFuture<'static, Outcome>;
}

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.PolicyKit1.Error")]
pub enum Error {
    Cancelled(String),
    Failed(String),
    #[zbus(error)]
    Transport(zbus::Error),
}
