pub mod anomaly;
pub mod evaluation;
pub mod holt;
#[cfg(test)]
mod predict_tests;
pub mod trace;
#[cfg(test)]
mod trace_tests;

pub use anomaly::Deviation;
pub use evaluation::{Forecaster, Persistence, Point, Score, evaluate};
pub use holt::Holt;
