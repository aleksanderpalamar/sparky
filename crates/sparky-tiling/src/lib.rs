pub mod desktop;
mod desktop_groups;
mod desktop_scratchpad;
mod desktop_windows;
mod desktop_workspaces;
pub mod drag;
pub mod drag_session;
pub mod dwindle;
pub mod focus;
pub mod gaps;
pub mod geometry;
pub mod groups;
pub mod ids;
pub mod neighbor;
mod node;
pub mod workspace;
pub mod workspace_groups;

#[cfg(test)]
mod desktop_group_tests;
#[cfg(test)]
mod desktop_scratchpad_tests;
#[cfg(test)]
mod desktop_tests;
#[cfg(test)]
mod desktop_window_tests;
#[cfg(test)]
mod drag_session_tests;
#[cfg(test)]
mod drag_tests;
#[cfg(test)]
mod dwindle_tests;
#[cfg(test)]
mod groups_tests;
#[cfg(test)]
mod workspace_tests;

pub use desktop::{Desktop, Monitor, Style};
pub use desktop_windows::Opening;
pub use desktop_workspaces::WorkspaceSummary;
pub use dwindle::Dwindle;
pub use gaps::Gaps;
pub use geometry::{Arrangement, Direction, Rect};
pub use ids::{WindowId, WorkspaceId};
pub use workspace::{Fullscreen, Layer, Placement, Screen, Workspace};
