use super::adam::Adam;
use super::mlp::Mlp;
use super::sequence::SequenceModel;

#[test]
fn a_mesma_semente_gera_o_mesmo_modelo() {
    assert_eq!(Mlp::new(&[4, 8, 2], 3), Mlp::new(&[4, 8, 2], 3));
    assert_ne!(Mlp::new(&[4, 8, 2], 3), Mlp::new(&[4, 8, 2], 4));
}

#[test]
fn a_mlp_aprende_rotulos_simultaneos() {
    let examples = [
        ([0.0, 0.0], [0.0, 0.0]),
        ([1.0, 0.0], [1.0, 0.0]),
        ([0.0, 1.0], [0.0, 1.0]),
        ([1.0, 1.0], [1.0, 1.0]),
    ];
    let mut model = Mlp::new(&[2, 8, 2], 5);
    let mut adam = Adam::new(0.05, model.params.len());
    for _ in 0..400 {
        for (input, target) in &examples {
            model.accumulate(input, target);
        }
        adam.step(&mut model.params);
    }
    for (input, target) in &examples {
        let predicted = model.predict(input);
        for (value, wanted) in predicted.iter().zip(target) {
            assert!((value - wanted).abs() < 0.2, "{input:?}: {predicted:?}");
        }
    }
}

#[test]
fn a_gru_aprende_a_continuar_uma_rampa() {
    let mut model = SequenceModel::new(1, 8, 1, 9);
    let mut adam = Adam::new(0.02, model.params.len());
    let window = |start: f32| {
        (0..6)
            .map(|step| vec![start + 0.05 * step as f32])
            .collect::<Vec<_>>()
    };
    let mut last = f32::MAX;
    for epoch in 0..300 {
        let mut total = 0.0;
        for start in [0.0, 0.1, 0.2, 0.3, 0.4] {
            total += model.accumulate(&window(start), &[Some(start + 0.05 * 8.0)]);
        }
        adam.step(&mut model.params);
        if epoch == 299 {
            last = total / 5.0;
        }
    }
    assert!(last < 1e-3, "erro final {last}");
    assert!(model.params.is_finite());
}
