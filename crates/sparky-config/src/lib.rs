pub mod action;
pub mod binds;
pub mod combo;
pub mod defaults;
pub mod edit;
pub mod load;
pub mod output;
pub mod schema;
pub mod screen;

#[cfg(test)]
mod defaults_tests;
#[cfg(test)]
mod edit_output_tests;
#[cfg(test)]
mod output_tests;
#[cfg(test)]
mod schema_tests;
#[cfg(test)]
mod screen_tests;

pub use action::{Action, Bind};
pub use edit::Editor;
pub use load::{ConfigError, parse, path};
pub use output::{OutputRule, OutputScale, rule_for, scale_for};
pub use schema::{Apps, Config, General, Input};
pub use screen::{OutputMode, Rotation};
