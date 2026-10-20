pub mod action;
pub mod binds;
pub mod combo;
pub mod defaults;
pub mod load;
pub mod schema;

#[cfg(test)]
mod schema_tests;

pub use action::{Action, Bind};
pub use load::{ConfigError, parse, path};
pub use schema::{Apps, Config, General, Input};
