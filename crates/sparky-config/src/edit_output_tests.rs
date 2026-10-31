use crate::edit::Editor;
use crate::load::parse;
use crate::output::{OutputRule, OutputScale, rule_for};
use crate::screen::{OutputMode, Rotation};

const ARQUIVO: &str = "# meu sparky\n[general]\nborder = 2\n\n# monitor da mesa\n[[output]]\nname = \"DP-1\"\nscale = 1.5\n";

fn regra(name: &str) -> OutputRule {
    OutputRule {
        name: name.to_owned(),
        scale: OutputScale::ONE,
        mode: Some(OutputMode {
            width: 1920,
            height: 1080,
            refresh: 60_000,
        }),
        rotation: Rotation::Left,
        vrr: Some(true),
        enabled: true,
    }
}

fn editado(regras: &[OutputRule]) -> String {
    let Ok(mut editor) = Editor::parse(ARQUIVO) else {
        panic!("arquivo de exemplo inválido")
    };
    for regra in regras {
        editor.set_output(regra);
    }
    editor.contents()
}

#[test]
fn atualiza_a_regra_do_monitor_e_guarda_o_resto() {
    let contents = editado(&[regra("DP-1")]);
    assert!(contents.contains("# meu sparky\n"));
    assert!(contents.contains("# monitor da mesa\n"));
    assert_eq!(contents.matches("[[output]]").count(), 1, "{contents}");
    let Ok(config) = parse(&contents) else {
        panic!("inválido: {contents}")
    };
    assert_eq!(rule_for(&config.output, "DP-1"), Some(&regra("DP-1")));
    assert_eq!(config.general.border, 2);
}

#[test]
fn monitor_novo_ganha_a_propria_regra() {
    let contents = editado(&[regra("HDMI-A-1")]);
    let Ok(config) = parse(&contents) else {
        panic!("inválido: {contents}")
    };
    assert_eq!(config.output.len(), 2);
    assert_eq!(
        rule_for(&config.output, "HDMI-A-1"),
        Some(&regra("HDMI-A-1"))
    );
    assert_eq!(
        rule_for(&config.output, "DP-1").map(|regra| regra.scale.factor()),
        Some(1.5)
    );
}

#[test]
fn valores_padrao_saem_do_arquivo() {
    let padrao = OutputRule {
        mode: None,
        rotation: Rotation::Normal,
        vrr: None,
        ..regra("DP-1")
    };
    let contents = editado(&[regra("DP-1"), padrao.clone()]);
    for chave in ["mode", "rotation", "vrr", "enabled"] {
        assert!(!contents.contains(chave), "{chave} sobrou: {contents}");
    }
    let desligada = OutputRule {
        enabled: false,
        ..padrao
    };
    assert!(editado(&[desligada]).contains("enabled = false"));
}
