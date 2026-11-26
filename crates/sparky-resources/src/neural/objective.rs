#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Objective {
    #[default]
    Squared,
    Huber(f32),
    Absolute,
}

pub fn measure(objective: Objective, predicted: &[f32], target: &[Option<f32>]) -> (f32, Vec<f32>) {
    let present = target.iter().flatten().count().max(1) as f32;
    let mut loss = 0.0;
    let gradient = predicted
        .iter()
        .zip(target)
        .map(|(value, wanted)| {
            let Some(wanted) = wanted else {
                return 0.0;
            };
            let (cost, slope) = term(objective, value - wanted);
            loss += cost / present;
            slope / present
        })
        .collect();
    (loss, gradient)
}

fn term(objective: Objective, error: f32) -> (f32, f32) {
    match objective {
        Objective::Squared => (error * error, 2.0 * error),
        Objective::Huber(delta) if error.abs() <= delta => (error * error / 2.0, error),
        Objective::Huber(delta) => (delta * (error.abs() - delta / 2.0), delta * sign(error)),
        Objective::Absolute => (error.abs(), sign(error)),
    }
}

fn sign(value: f32) -> f32 {
    match value {
        value if value > 0.0 => 1.0,
        value if value < 0.0 => -1.0,
        _ => 0.0,
    }
}
