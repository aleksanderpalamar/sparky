use super::objective::{Objective, measure};
use crate::application::learning::Plan;

const CASES: [Objective; 3] = [
    Objective::Squared,
    Objective::Huber(0.02),
    Objective::Absolute,
];

#[test]
fn quadratico_continua_igual_ao_de_antes() {
    let (loss, gradient) = measure(Objective::Squared, &[0.5, 0.2], &[Some(0.3), None]);
    assert!((loss - 0.04).abs() < 1e-6);
    assert!((gradient[0] - 0.4).abs() < 1e-6);
    assert_eq!(gradient[1], 0.0);
}

#[test]
fn huber_e_quadratico_perto_e_linear_longe() {
    let (_, near) = measure(Objective::Huber(0.02), &[0.01], &[Some(0.0)]);
    let (_, far) = measure(Objective::Huber(0.02), &[0.1], &[Some(0.0)]);
    assert!((near[0] - 0.01).abs() < 1e-6);
    assert!((far[0] - 0.02).abs() < 1e-6);
}

#[test]
fn absoluto_usa_so_o_sinal_do_erro() {
    let (loss, gradient) = measure(
        Objective::Absolute,
        &[0.3, 0.1, 0.2],
        &[Some(0.1), Some(0.3), Some(0.2)],
    );
    assert!((loss - 0.4 / 3.0).abs() < 1e-6);
    assert!((gradient[0] - 1.0 / 3.0).abs() < 1e-6);
    assert!((gradient[1] + 1.0 / 3.0).abs() < 1e-6);
    assert_eq!(gradient[2], 0.0);
}

#[test]
fn gradiente_confere_com_a_derivada_numerica() {
    let target = [Some(0.25), Some(-0.4)];
    let point = [0.31, -0.37];
    for objective in CASES {
        let (_, gradient) = measure(objective, &point, &target);
        for slot in 0..point.len() {
            let step = 1e-3;
            let mut up = point;
            let mut down = point;
            up[slot] += step;
            down[slot] -= step;
            let numeric = (measure(objective, &up, &target).0
                - measure(objective, &down, &target).0)
                / (2.0 * step);
            assert!(
                (numeric - gradient[slot]).abs() < 1e-3,
                "{objective:?} saida {slot}: {numeric} x {}",
                gradient[slot]
            );
        }
    }
}

#[test]
fn producao_continua_quadratica_com_decay_de_dez_por_cento() {
    let plan = Plan::default();
    assert_eq!(plan.objective, Objective::Squared);
    assert_eq!(plan.decay, 0.1);
}
