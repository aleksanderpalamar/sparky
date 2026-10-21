use crate::load::parse;
use crate::output::{OutputScale, scale_for};

#[test]
fn le_a_escala_fracionada_de_cada_monitor() {
    let contents = "[[output]]\nname = \"DP-1\"\nscale = 1.5\n\n[[output]]\nname = \"HDMI-A-1\"\nscale = 2\n";
    let Ok(config) = parse(contents) else { panic!("deveria ler os monitores") };
    assert_eq!(scale_for(&config.output, "DP-1").factor(), 1.5);
    assert_eq!(scale_for(&config.output, "HDMI-A-1").factor(), 2.0);
}

#[test]
fn monitor_sem_regra_fica_em_um() {
    assert_eq!(scale_for(&[], "DP-1"), OutputScale::ONE);
    let Ok(config) = parse("[[output]]\nname = \"DP-1\"\n") else { panic!("escala é opcional") };
    assert_eq!(scale_for(&config.output, "DP-1"), OutputScale::ONE);
}

#[test]
fn a_escala_e_guardada_em_cento_e_vinte_avos() {
    assert_eq!(OutputScale::from_factor(1.25).map(OutputScale::units), Some(150));
    assert_eq!(OutputScale::from_factor(1.0), Some(OutputScale::ONE));
}

#[test]
fn escala_fora_de_meio_a_quatro_e_recusada() {
    assert_eq!(OutputScale::from_factor(0.3), None);
    assert_eq!(OutputScale::from_factor(5.0), None);
    assert!(parse("[[output]]\nname = \"DP-1\"\nscale = 9\n").is_err());
}

#[test]
fn a_ultima_regra_do_mesmo_monitor_vale() {
    let contents = "[[output]]\nname = \"DP-1\"\nscale = 1.5\n\n[[output]]\nname = \"DP-1\"\nscale = 1.25\n";
    let Ok(config) = parse(contents) else { panic!("deveria ler") };
    assert_eq!(scale_for(&config.output, "DP-1").factor(), 1.25);
}
