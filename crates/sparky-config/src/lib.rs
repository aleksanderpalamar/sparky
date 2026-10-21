pub mod action;
pub mod binds;
pub mod combo;
pub mod defaults;
pub mod edit;
pub mod load;
pub mod schema;

#[cfg(test)]
mod defaults_tests;
#[cfg(test)]
mod schema_tests;

pub use action::{Action, Bind};
pub use edit::Editor;
pub use load::{ConfigError, parse, path};
pub use schema::{Apps, Config, General, Input};
