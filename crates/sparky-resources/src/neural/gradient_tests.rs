use super::mlp::Mlp;
use super::params::Parameters;
use super::sequence::SequenceModel;

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
fn gradiente_da_mlp_bate_com_diferencas_finitas() {
    let mut model = Mlp::new(&[3, 5, 4, 2], 7);
    let (input, target) = ([0.3, -0.8, 0.5], [1.0, 0.0]);
    model.params.zero_grads();
    model.accumulate(&input, &target);
    let analytic = model.params.grads.clone();
    compare(&analytic, |index| {
        let probe = |delta| {
            let mut copy = model.clone();
            copy.params = nudged(&model.params, index, delta);
            copy.loss(&input, &target)
        };
        (probe(STEP) - probe(-STEP)) / (2.0 * STEP)
    });
}

#[test]
fn gradiente_da_gru_atravessa_o_tempo_corretamente() {
    let mut model = SequenceModel::new(2, 3, 2, 11);
    let sequence = vec![
        vec![0.5, -0.2],
        vec![0.1, 0.9],
        vec![-0.7, 0.3],
        vec![0.4, 0.4],
    ];
    let target = [Some(0.25), None];
    model.params.zero_grads();
    model.accumulate(&sequence, &target);
    let analytic = model.params.grads.clone();
    compare(&analytic, |index| {
        let probe = |delta| {
            let mut copy = model.clone();
            copy.params = nudged(&model.params, index, delta);
            copy.loss(&sequence, &target)
        };
        (probe(STEP) - probe(-STEP)) / (2.0 * STEP)
    });
}
