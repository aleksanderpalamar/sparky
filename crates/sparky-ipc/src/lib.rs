pub mod client;
#[cfg(test)]
mod client_tests;
pub mod codec;
pub mod command;
#[cfg(test)]
mod command_tests;
pub mod error;
pub mod info;
pub mod message;
#[cfg(test)]
mod message_tests;
pub mod output;
pub mod socket;

pub use client::{Client, Events};
pub use error::IpcError;
pub use info::{SessionInfo, SessionKind};
pub use message::{CaptureArea, Event, IdleHold, Reply, Request, SessionAction, WorkspaceInfo};
pub use output::{Hdr, OutputInfo, Vrr};
