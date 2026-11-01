use sparky_config::{OutputMode, OutputScale};
use sparky_ipc::{OutputInfo, Vrr};
use sparky_ui::notice::Notice;

use crate::app::message::Message;
use crate::app::pickers::ScreenField;
use crate::app::state::Settings;
use crate::app::test_support::{Arquivo, Sparky, painel, salvo};
use crate::app::update::update;
use crate::domain::screens::ScreenChange;
use crate::domain::section::Section;

const NOME: &str = "HDMI-A-1";

fn monitor() -> OutputInfo {
    let modos = vec![
        OutputMode {
            width: 1920,
            height: 1080,
            refresh: 60_000,
        },
        OutputMode {
            width: 1280,
            height: 720,
            refresh: 60_000,
        },
    ];
    OutputInfo {
        name: NOME.to_owned(),
        width: 1920,
        height: 1080,
        mode: modos.first().copied(),
        modes: modos,
        scale: OutputScale::ONE,
        vrr: Vrr::Off,
        enabled: true,
        ..OutputInfo::default()
    }
}

fn abrir() -> (Settings, Arquivo, Sparky) {
    let sparky = Sparky {
        monitores: vec![monitor()],
        ..Sparky::default()
    };
    let (mut settings, arquivo) = painel(None, sparky.clone());
    let _ = update(&mut settings, Message::Section(Section::Monitors));
    (settings, arquivo, sparky)
}

fn mudar(settings: &mut Settings, change: ScreenChange) {
    let _ = update(settings, Message::Screen(NOME.to_owned(), change));
}

#[test]
fn cada_monitor_tem_as_proprias_listas() {
    let (settings, _, _) = abrir();
    let opcoes = settings
        .pickers
        .screen(NOME, ScreenField::Mode)
        .map(|state| state.options().to_vec());
    assert_eq!(
        opcoes,
        Some(vec![
            "1920x1080@60.00Hz".to_owned(),
            "1280x720@60.00Hz".to_owned()
        ])
    );
    assert!(
        settings
            .pickers
            .screen(NOME, ScreenField::Rotation)
            .is_some()
    );
}

#[test]
fn escolher_grava_a_regra_do_monitor_e_recarrega() {
    let (mut settings, arquivo, sparky) = abrir();
    mudar(&mut settings, ScreenChange::Scale("1.5".to_owned()));
    mudar(
        &mut settings,
        ScreenChange::Mode("1280x720@60.00Hz".to_owned()),
    );
    mudar(
        &mut settings,
        ScreenChange::Rotation("90° à esquerda".to_owned()),
    );
    mudar(&mut settings, ScreenChange::Vrr(true));
    let contents = salvo(&arquivo);
    for trecho in [
        "[[output]]",
        "name = \"HDMI-A-1\"",
        "scale = 1.5",
        "mode = \"1280x720@60.000\"",
        "rotation = 270",
        "vrr = true",
    ] {
        assert!(contents.contains(trecho), "faltou {trecho}: {contents}");
    }
    assert_eq!(contents.matches("[[output]]").count(), 1);
    assert_eq!(sparky.recargas.get(), 4);
}

#[test]
fn a_unica_tela_ligada_nao_e_desligada() {
    let (mut settings, arquivo, sparky) = abrir();
    mudar(&mut settings, ScreenChange::Enabled(false));
    assert!(matches!(&settings.notice, Some(Notice::Error(text)) if text.contains("única tela")));
    assert!(salvo(&arquivo).is_empty());
    assert_eq!(sparky.recargas.get(), 0);
}
