use serde::{Deserialize, Serialize};

use crate::domain::{Features, METRICS, Metric};
use crate::predict::{Deviation, Holt};

pub const SHORT: f64 = 10.0;
pub const LONG: f64 = 60.0;
pub const ALPHA: f32 = 0.2;
pub const BETA: f32 = 0.0;
const DEVIATION_WEIGHT: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Insight {
    pub metric: Metric,
    pub current: Option<f32>,
    pub estimate_short: Option<f32>,
    pub estimate_long: Option<f32>,
    pub anomaly: Option<f32>,
    pub neural_short: Option<f32>,
    pub neural_long: Option<f32>,
}

pub struct Baseline {
    forecasters: [Holt; METRICS.len()],
    deviations: [Deviation; METRICS.len()],
}

impl Default for Baseline {
    fn default() -> Self {
        Self {
            forecasters: [Holt::new(ALPHA, BETA); METRICS.len()],
            deviations: [Deviation::new(DEVIATION_WEIGHT); METRICS.len()],
        }
    }
}

impl Baseline {
    pub fn observe(&mut self, at: f64, features: &Features) -> Vec<Insight> {
        METRICS
            .iter()
            .map(|metric| self.metric(at, *metric, features.get(*metric)))
            .collect()
    }

    fn metric(&mut self, at: f64, metric: Metric, current: Option<f32>) -> Insight {
        let index = metric as usize;
        let anomaly = current.and_then(|value| self.deviations[index].score(value));
        if let Some(value) = current {
            self.forecasters[index].update(at, value);
            self.deviations[index].update(value);
        }
        let estimate = |ahead| {
            self.forecasters[index]
                .forecast(ahead)
                .map(|value| value.clamp(0.0, 100.0))
        };
        Insight {
            metric,
            current,
            estimate_short: estimate(SHORT),
            estimate_long: estimate(LONG),
            anomaly,
            neural_short: None,
            neural_long: None,
        }
    }
}
