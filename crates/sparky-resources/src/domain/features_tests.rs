use super::features::{Features, Metric};

#[test]
fn guarda_cada_metrica_em_percentual() {
    let features = Features::default()
        .with(Metric::CpuBusy, Some(42.5))
        .with(Metric::GpuBusy, None);
    assert_eq!(features.get(Metric::CpuBusy), Some(42.5));
    assert_eq!(features.get(Metric::GpuBusy), None);
    assert_eq!(features.get(Metric::VramUsed), None);
}

#[test]
fn valores_fora_da_faixa_ou_invalidos_nao_entram_como_medicao() {
    let features = Features::default()
        .with(Metric::MemoryUsed, Some(140.0))
        .with(Metric::IoPressure, Some(-3.0))
        .with(Metric::CpuPressure, Some(f32::NAN));
    assert_eq!(features.get(Metric::MemoryUsed), Some(100.0));
    assert_eq!(features.get(Metric::IoPressure), Some(0.0));
    assert_eq!(features.get(Metric::CpuPressure), None);
}
