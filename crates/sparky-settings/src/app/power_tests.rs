use sparky_ui::notice::Notice;

use crate::app::state::Settings;
use crate::app::test_support::{Sparky, painel, salvo, tecla};
use crate::domain::section::Section;
use crate::domain::system::RowId;

fn energia(conteudo: Option<&str>) -> (Settings, crate::app::test_support::Arquivo) {
    let (mut settings, arquivo) = painel(conteudo, Sparky::default());
    settings.go(Section::Power);
    (settings, arquivo)
}

fn valor(settings: &Settings) -> Option<String> {
    settings
        .system_rows()
        .into_iter()
        .find(|row| row.id == RowId::IdleLock)
        .map(|row| row.value)
}

fn focar(settings: &mut Settings) {
    let Some(index) = settings
        .system_rows()
        .iter()
        .position(|row| row.id == RowId::IdleLock)
    else {
        panic!("a seção Energia deveria ter a linha do bloqueio");
    };
    settings.point(index);
}

#[test]
fn a_secao_de_energia_mostra_o_bloqueio_por_inatividade() {
    let (settings, _) = energia(None);
    assert_eq!(valor(&settings).as_deref(), Some("10 minutos"));
    let (settings, _) = energia(Some("[lock]\nidle_minutes = 0\n"));
    assert_eq!(valor(&settings).as_deref(), Some("Nunca"));
}

#[test]
fn as_setas_mudam_o_tempo_e_gravam_no_sparky_toml() {
    let (mut settings, arquivo) = energia(None);
    focar(&mut settings);
    tecla(&mut settings, "l");
    assert_eq!(settings.config.lock.idle_minutes, 15);
    assert!(salvo(&arquivo).contains("[lock]\nidle_minutes = 15"));
    assert_eq!(valor(&settings).as_deref(), Some("15 minutos"));
    assert!(matches!(
        &settings.notice,
        Some(Notice::Info(text)) if text.starts_with("Bloquear por inatividade: 15 minutos")
    ));
}

#[test]
fn escolher_nunca_desliga_o_bloqueio_por_inatividade() {
    let (mut settings, arquivo) = energia(None);
    settings.choose(RowId::IdleLock, "Nunca".to_owned());
    assert_eq!(settings.config.lock.idle_minutes, 0);
    assert!(salvo(&arquivo).contains("[lock]\nidle_minutes = 0"));
}
