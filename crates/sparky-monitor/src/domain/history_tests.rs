use super::history::{History, SAMPLES};

#[test]
fn guarda_somente_os_ultimos_sessenta_valores() {
    let mut history = History::default();
    for value in 0..70 {
        history.push(value as f32);
    }
    let values: Vec<_> = history.values().collect();
    assert_eq!(values.len(), SAMPLES);
    assert_eq!(values.first(), Some(&10.0));
    assert_eq!(history.latest(), Some(69.0));
    assert_eq!(history.peak(), 69.0);
}

#[test]
fn historico_vazio_tem_pico_zero() {
    let history = History::default();
    assert_eq!(history.latest(), None);
    assert_eq!(history.peak(), 0.0);
}
