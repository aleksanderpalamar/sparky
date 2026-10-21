pub mod client;
pub mod codec;
pub mod command;
pub mod error;
pub mod message;
pub mod socket;

pub use client::{Client, Events};
pub use error::IpcError;
pub use message::{CaptureArea, Event, OutputInfo, Reply, Request, SessionAction, WorkspaceInfo};
