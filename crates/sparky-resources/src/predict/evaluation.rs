use super::holt::Holt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub at: f64,
    pub value: f32,
}

pub trait Forecaster {
    fn update(&mut self, at: f64, value: f32);
    fn forecast(&self, ahead: f64) -> Option<f32>;
}

impl Forecaster for Holt {
    fn update(&mut self, at: f64, value: f32) {
        Holt::update(self, at, value);
    }

    fn forecast(&self, ahead: f64) -> Option<f32> {
        Holt::forecast(self, ahead)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Persistence {
    last: Option<f32>,
}

impl Forecaster for Persistence {
    fn update(&mut self, _at: f64, value: f32) {
        self.last = Some(value);
    }

    fn forecast(&self, _ahead: f64) -> Option<f32> {
        self.last
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Score {
    pub mean_absolute_error: f32,
    pub forecasts: usize,
}

pub fn evaluate(points: &[Point], model: &mut impl Forecaster, horizon: f64) -> Option<Score> {
    let tolerance = horizon * 0.5;
    let mut target = 0;
    let mut total = 0.0_f64;
    let mut forecasts = 0;
    for (index, point) in points.iter().enumerate() {
        model.update(point.at, point.value);
        target = target.max(index + 1);
        while points
            .get(target)
            .is_some_and(|future| future.at < point.at + horizon)
        {
            target += 1;
        }
        let Some(future) = points
            .get(target)
            .filter(|future| future.at <= point.at + horizon + tolerance)
        else {
            continue;
        };
        let Some(predicted) = model.forecast(future.at - point.at) else {
            continue;
        };
        total += f64::from((predicted.clamp(0.0, 100.0) - future.value).abs());
        forecasts += 1;
    }
    (forecasts > 0).then(|| Score {
        mean_absolute_error: (total / forecasts as f64) as f32,
        forecasts,
    })
}
