pub mod action;
pub mod binds;
pub mod combo;
pub mod defaults;
pub mod dock;
pub mod edit;
pub mod load;
pub mod lock;
pub mod output;
mod renamed;
pub mod resources;
pub mod schema;
pub mod screen;
pub mod terminal;

#[cfg(test)]
mod defaults_tests;
#[cfg(test)]
mod dock_tests;
#[cfg(test)]
mod edit_output_tests;
#[cfg(test)]
mod glass_tests;
#[cfg(test)]
mod output_tests;
#[cfg(test)]
mod renamed_tests;
#[cfg(test)]
mod resources_tests;
#[cfg(test)]
mod schema_tests;
#[cfg(test)]
mod screen_tests;

pub use action::{Action, Bind};
pub use dock::Dock;
pub use edit::Editor;
pub use load::{ConfigError, parse, path};
pub use lock::Lock;
pub use output::{OutputRule, OutputScale, rule_for, scale_for};
pub use renamed::migrate;
pub use resources::{ResourceMode, Resources};
pub use schema::{Apps, Config, General, Input, Login};
pub use screen::{OutputMode, Rotation};
pub use terminal::{Greeting, Terminal};
