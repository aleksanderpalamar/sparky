use super::adam::Adam;
use super::hybrid::{Hybrid, Shape, activate, slope};

fn shape(context: usize) -> Shape {
    Shape {
        steps: 1,
        context,
        memory: 8,
        hidden: 8,
        outputs: 1,
    }
}

fn ramp(start: f32) -> Vec<Vec<f32>> {
    (0..6)
        .map(|step| vec![start + 0.05 * step as f32])
        .collect()
}

#[test]
fn a_mesma_semente_gera_o_mesmo_modelo() {
    assert_eq!(Hybrid::new(shape(2), 3), Hybrid::new(shape(2), 3));
    assert_ne!(Hybrid::new(shape(2), 3), Hybrid::new(shape(2), 4));
}

#[test]
fn o_hibrido_aprende_a_continuar_uma_rampa() {
    let mut model = Hybrid::new(shape(0), 9);
    let mut adam = Adam::new(0.02, model.params.len());
    let mut last = f32::MAX;
    for epoch in 0..300 {
        let mut total = 0.0;
        for start in [0.0, 0.1, 0.2, 0.3, 0.4] {
            total += model.accumulate(&ramp(start), &[], &[Some(start + 0.05 * 8.0)]);
        }
        adam.step(&mut model.params);
        if epoch == 299 {
            last = total / 5.0;
        }
    }
    assert!(last < 1e-3, "erro final {last}");
    assert!(model.params.is_finite());
}

#[test]
fn o_contexto_muda_a_previsao_para_a_mesma_serie() {
    let mut model = Hybrid::new(shape(2), 5);
    let mut adam = Adam::new(0.02, model.params.len());
    let series = ramp(0.2);
    let cases = [([1.0, 0.0], 0.1), ([0.0, 1.0], -0.3)];
    for _ in 0..400 {
        for (context, wanted) in &cases {
            model.accumulate(&series, context, &[Some(*wanted)]);
        }
        adam.step(&mut model.params);
    }
    for (context, wanted) in &cases {
        let predicted = model.predict(&series, context)[0];
        assert!(
            (predicted - wanted).abs() < 0.05,
            "{context:?}: {predicted}"
        );
    }
}

#[test]
fn modelo_parado_comeca_prevendo_zero_e_ainda_aprende() {
    let mut model = Hybrid::still(shape(1), 9);
    let sequence = [vec![0.2], vec![0.4]];
    assert_eq!(model.predict(&sequence, &[1.0]), vec![0.0]);
    let mut adam = Adam::new(0.01, model.params.len());
    let before = model.loss(&sequence, &[1.0], &[Some(0.5)]);
    for _ in 0..50 {
        model.accumulate(&sequence, &[1.0], &[Some(0.5)]);
        adam.step(&mut model.params);
    }
    assert!(model.loss(&sequence, &[1.0], &[Some(0.5)]) < before * 0.1);
}

#[test]
fn a_camada_oculta_nao_tem_regiao_sem_gradiente() {
    for sum in [-3.0, -1.0, -0.1, 0.0, 0.1, 1.0, 3.0] {
        assert!(slope(activate(sum)) > 0.0, "soma {sum}");
    }
}

#[test]
fn acomodar_puxa_a_saida_de_volta_para_zero() {
    let mut model = Hybrid::still(shape(1), 9);
    let mut adam = Adam::new(0.05, model.params.len());
    let sequence = [vec![0.2], vec![0.4]];
    for _ in 0..50 {
        model.accumulate(&sequence, &[1.0], &[Some(0.5)]);
        adam.step(&mut model.params);
    }
    let learned = model.predict(&sequence, &[1.0])[0];
    let mut kept = model.clone();
    kept.settle(0.0);
    assert_eq!(kept.predict(&sequence, &[1.0])[0], learned);
    model.settle(0.5);
    let halved = model.predict(&sequence, &[1.0])[0];
    assert!((halved - learned * 0.5).abs() < 1e-6, "{halved} {learned}");
    model.settle(1.0);
    assert_eq!(model.predict(&sequence, &[1.0]), vec![0.0]);
}

#[test]
fn alargar_o_contexto_mantem_exatamente_o_que_a_rede_previa() {
    let mut model = Hybrid::new(shape(2), 5);
    let mut adam = Adam::new(0.02, model.params.len());
    for _ in 0..30 {
        model.accumulate(&ramp(0.1), &[1.0, 0.0], &[Some(0.3)]);
        adam.step(&mut model.params);
    }
    let Some(wider) = model.widen(shape(5)) else {
        panic!("deveria alargar")
    };
    for extra in [[0.0, 0.0, 0.0], [0.7, -0.4, 1.0]] {
        let context = [1.0, 0.0, extra[0], extra[1], extra[2]];
        assert_eq!(
            wider.predict(&ramp(0.1), &context),
            model.predict(&ramp(0.1), &[1.0, 0.0])
        );
    }
    assert!(
        model
            .widen(Shape {
                memory: 4,
                ..shape(5)
            })
            .is_none()
    );
    assert!(model.widen(shape(1)).is_none());
}
