use super::reading::Reading;

#[test]
fn leitura_recente_e_medida() {
    assert_eq!(
        Reading::observed(42, 500, 2_000),
        Reading::Measured {
            value: 42,
            age_ms: 500
        }
    );
}

#[test]
fn leitura_antiga_e_obsoleta() {
    assert_eq!(
        Reading::observed(42, 2_001, 2_000),
        Reading::Stale {
            value: 42,
            age_ms: 2_001
        }
    );
}

#[test]
fn so_a_medida_vale_como_medicao() {
    assert_eq!(Reading::observed(7, 0, 10).measured(), Some(&7));
    assert_eq!(Reading::Estimated { value: 7 }.measured(), None);
    assert_eq!(Reading::<u8>::Missing.measured(), None);
    assert_eq!(Reading::observed(7, 11, 10).measured(), None);
}

#[test]
fn a_origem_do_valor_vai_no_json() {
    let json = serde_json::to_string(&Reading::Estimated { value: 1.5 }).unwrap_or_default();
    assert_eq!(json, r#"{"kind":"estimated","value":1.5}"#);
    let missing = serde_json::to_string(&Reading::<f32>::Missing).unwrap_or_default();
    assert_eq!(missing, r#"{"kind":"missing"}"#);
}
