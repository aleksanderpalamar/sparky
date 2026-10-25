use super::model::{Selection, Source};

#[test]
fn a_escolha_preserva_a_linha_do_portal() {
    let raw = "Window: Euro Truck Simulator 2 (abc-123)";
    let source = Source::new(raw);
    assert_eq!(source.label, "Janela: Euro Truck Simulator 2");
    assert_eq!(source.raw, raw);
    assert_eq!(
        Source::new("Monitor: DP-1 Dell").label,
        "Monitor: DP-1 Dell"
    );
    assert_eq!(Source::new("HDMI-A-1").raw, "HDMI-A-1");
}

#[test]
fn busca_e_navegacao_nao_escolhem_uma_fonte_oculta() {
    let mut selection =
        Selection::new("Monitor: DP-1 Dell\nWindow: ETS2 (abc)\nWindow: Terminal (def)\n");
    selection.search("ets2".into());
    assert_eq!(
        selection.chosen().map(|s| s.raw.as_str()),
        Some("Window: ETS2 (abc)")
    );
    selection.step(1);
    assert_eq!(selection.visible().len(), 1);
    selection.search("inexistente".into());
    assert!(selection.chosen().is_none());
    selection.step(-1);
    assert!(selection.chosen().is_none());
}

#[test]
fn setas_circulam_e_entrada_vazia_cancela() {
    let mut selection = Selection::new("a\nb\n");
    selection.step(-1);
    assert_eq!(selection.chosen().map(|s| s.raw.as_str()), Some("b"));
    selection.step(1);
    assert_eq!(selection.chosen().map(|s| s.raw.as_str()), Some("a"));
    assert!(Selection::new("\n\n").chosen().is_none());
}

#[test]
fn titulos_com_parenteses_e_controles_preservam_o_identificador() {
    let source = Source::new("Window: Terminal (projeto)\t (xyz)");
    assert_eq!(source.label, "Janela: Terminal (projeto)");
    assert_eq!(source.raw, "Window: Terminal (projeto)\t (xyz)");
}
