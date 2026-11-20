pub mod names;
pub mod recognizer;
#[cfg(test)]
mod recognizer_tests;
pub mod scene;

pub use recognizer::recognize;
pub use scene::{Load, Process, Scene, Window};
