use sparky_config::{OutputMode, OutputRule, OutputScale, Rotation};
use sparky_ipc::{Hdr, OutputInfo, Vrr};

use super::screens::{
    ScreenChange, changed, mode_options, rotation_label, rotation_options, scale_label,
    scale_options,
};

const MODOS: [OutputMode; 3] = [
    OutputMode {
        width: 2560,
        height: 1440,
        refresh: 143_855,
    },
    OutputMode {
        width: 2560,
        height: 1440,
        refresh: 143_855,
    },
    OutputMode {
        width: 1920,
        height: 1080,
        refresh: 60_000,
    },
];

fn monitor(name: &str, vrr: Vrr) -> OutputInfo {
    OutputInfo {
        name: name.to_owned(),
        mode: Some(MODOS[0]),
        modes: MODOS.to_vec(),
        scale: OutputScale::from_factor(1.25).unwrap_or_default(),
        vrr,
        enabled: true,
        ..OutputInfo::default()
    }
}

#[test]
fn opcoes_de_resolucao_sem_repeticao() {
    assert_eq!(
        mode_options(&monitor("DP-1", Vrr::Off)),
        ["2560x1440@143.86Hz", "1920x1080@60.00Hz"]
    );
}

#[test]
fn escalas_comuns_mais_a_atual() {
    assert_eq!(
        scale_options(OutputScale::ONE),
        ["1", "1.25", "1.5", "1.75", "2", "2.5"]
    );
    let incomum = OutputScale::from_factor(1.6).unwrap_or_default();
    assert_eq!(
        scale_options(incomum),
        ["1", "1.25", "1.5", "1.6", "1.75", "2", "2.5"]
    );
    assert_eq!(scale_label(incomum), "1.6");
}

#[test]
fn giros_com_nomes_da_referencia() {
    assert_eq!(
        rotation_options(),
        ["Normal", "90° à direita", "180°", "90° à esquerda"]
    );
    assert_eq!(rotation_label(Rotation::Left), "90° à esquerda");
}

#[test]
fn cada_escolha_vira_a_regra_do_monitor() {
    let dp = monitor("DP-1", Vrr::Off);
    let todos = [dp.clone()];
    let regra = |change| changed(&dp, &[], &todos, change);
    let Ok(modo) = regra(ScreenChange::Mode("1920x1080@60.00Hz".to_owned())) else {
        panic!("modo válido")
    };
    assert_eq!(modo.mode, Some(MODOS[2]));
    assert_eq!(modo.scale.factor(), 1.25);
    let Ok(escala) = regra(ScreenChange::Scale("2".to_owned())) else {
        panic!("escala válida")
    };
    assert_eq!(escala.scale.factor(), 2.0);
    let Ok(giro) = regra(ScreenChange::Rotation("90° à direita".to_owned())) else {
        panic!("giro válido")
    };
    assert_eq!(giro.rotation, Rotation::Right);
    let Ok(vrr) = regra(ScreenChange::Vrr(true)) else {
        panic!("vrr suportado")
    };
    assert_eq!(vrr.vrr, Some(true));
}

#[test]
fn a_regra_gravada_e_mantida_ao_mudar_outra_opcao() {
    let dp = monitor("DP-1", Vrr::Off);
    let gravada = OutputRule {
        rotation: Rotation::Left,
        ..OutputRule::new("DP-1")
    };
    let Ok(regra) = changed(
        &dp,
        &[gravada],
        std::slice::from_ref(&dp),
        ScreenChange::Scale("1.5".to_owned()),
    ) else {
        panic!("escala válida")
    };
    assert_eq!(regra.rotation, Rotation::Left);
    assert_eq!(regra.scale.factor(), 1.5);
}

#[test]
fn escolhas_impossiveis_sao_recusadas() {
    let dp = monitor("DP-1", Vrr::Unsupported);
    let todos = [dp.clone()];
    let regra = |change| changed(&dp, &[], &todos, change);
    assert!(regra(ScreenChange::Mode("3840x2160@60.00Hz".to_owned())).is_err());
    assert!(regra(ScreenChange::Scale("9".to_owned())).is_err());
    assert!(regra(ScreenChange::Rotation("45°".to_owned())).is_err());
    assert!(regra(ScreenChange::Vrr(true)).is_err());
}

#[test]
fn a_unica_tela_ligada_nao_pode_ser_desligada() {
    let dp = monitor("DP-1", Vrr::Off);
    let hdmi = OutputInfo {
        enabled: false,
        ..monitor("HDMI-A-1", Vrr::Off)
    };
    let todos = [dp.clone(), hdmi.clone()];
    assert!(changed(&dp, &[], &todos, ScreenChange::Enabled(false)).is_err());
    let Ok(religada) = changed(&hdmi, &[], &todos, ScreenChange::Enabled(true)) else {
        panic!("religar")
    };
    assert!(religada.enabled);
    let ligadas = [
        dp.clone(),
        OutputInfo {
            enabled: true,
            ..hdmi
        },
    ];
    let Ok(desligada) = changed(&dp, &[], &ligadas, ScreenChange::Enabled(false)) else {
        panic!("desligar")
    };
    assert!(!desligada.enabled);
}

#[test]
fn hdr_so_liga_no_monitor_que_suporta() {
    let suportado = OutputInfo {
        hdr: Hdr::Off,
        ..monitor("HDMI-A-1", Vrr::Unsupported)
    };
    let Ok(ligado) = changed(
        &suportado,
        &[],
        std::slice::from_ref(&suportado),
        ScreenChange::Hdr(true),
    ) else {
        panic!("HDR suportado")
    };
    assert!(ligado.hdr);
    let sem_hdr = monitor("DP-1", Vrr::Off);
    let erro = changed(
        &sem_hdr,
        &[],
        std::slice::from_ref(&sem_hdr),
        ScreenChange::Hdr(true),
    );
    assert!(erro.is_err_and(|erro| erro.contains("HDR")));
}
