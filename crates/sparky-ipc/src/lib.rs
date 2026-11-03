pub mod client;
pub mod codec;
pub mod command;
#[cfg(test)]
mod command_tests;
pub mod error;
pub mod message;
pub mod output;
pub mod socket;

pub use client::{Client, Events};
pub use error::IpcError;
pub use message::{CaptureArea, Event, Reply, Request, SessionAction, WorkspaceInfo};
pub use output::{Hdr, OutputInfo, Vrr};
