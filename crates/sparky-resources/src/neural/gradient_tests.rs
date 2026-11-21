use super::hybrid::{Hybrid, Shape};
use super::params::Parameters;

const STEP: f32 = 1e-3;
const TOLERANCE: f32 = 2e-2;

fn compare(analytic: &[f32], numeric: impl Fn(usize) -> f32) {
    for (index, expected) in analytic.iter().enumerate() {
        let estimate = numeric(index);
        let scale = expected.abs().max(estimate.abs()).max(1e-2);
        assert!(
            (expected - estimate).abs() / scale < TOLERANCE,
            "parâmetro {index}: analítico {expected}, numérico {estimate}"
        );
    }
}

fn nudged(params: &Parameters, index: usize, delta: f32) -> Parameters {
    let mut copy = params.clone();
    copy.values[index] += delta;
    copy
}

#[test]
fn gradiente_do_hibrido_atravessa_o_mlp_e_o_tempo() {
    let shape = Shape {
        steps: 2,
        context: 2,
        memory: 3,
        hidden: 4,
        outputs: 2,
    };
    let mut model = Hybrid::new(shape, 11);
    let sequence = vec![
        vec![0.5, -0.2],
        vec![0.1, 0.9],
        vec![-0.7, 0.3],
        vec![0.4, 0.4],
    ];
    let context = [1.0, 0.0];
    let target = [Some(0.25), None];
    model.params.zero_grads();
    model.accumulate(&sequence, &context, &target);
    let analytic = model.params.grads.clone();
    compare(&analytic, |index| {
        let probe = |delta| {
            let mut copy = model.clone();
            copy.params = nudged(&model.params, index, delta);
            copy.loss(&sequence, &context, &target)
        };
        (probe(STEP) - probe(-STEP)) / (2.0 * STEP)
    });
}
