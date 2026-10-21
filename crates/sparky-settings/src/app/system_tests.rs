use std::rc::Rc;

use sparky_ui::notice::Notice;

use crate::app::test_support::{Comandos, Sparky, painel_com, tecla};
use crate::domain::section::Section;
use crate::ports::fake::FakeCommands;

const PERFIS: &str = "* performance:\n    CpuDriver:\tamd_pstate\n\n  balanced:\n    CpuDriver:\tamd_pstate\n";

fn comandos(fake: FakeCommands) -> Comandos {
    Comandos(Rc::new(fake))
}

#[test]
fn a_secao_de_energia_mostra_o_perfil_e_troca_pelas_setas() {
    let shared = comandos(FakeCommands::new().with("powerprofilesctl list", PERFIS).accepting("powerprofilesctl set balanced"));
    let (mut settings, _) = painel_com(None, Sparky::default(), shared.clone());
    tecla(&mut settings, "7");
    assert_eq!(settings.section, Section::Power);
    assert_eq!(settings.count(), 1);
    tecla(&mut settings, "l");
    assert!(shared.0.calls().contains(&"powerprofilesctl set balanced".to_owned()));
    assert!(matches!(&settings.notice, Some(Notice::Info(text)) if text.starts_with("Perfil de energia")));
}

#[test]
fn recusa_do_sistema_vira_aviso_de_erro() {
    let shared = comandos(FakeCommands::new().with("powerprofilesctl list", PERFIS));
    let (mut settings, _) = painel_com(None, Sparky::default(), shared);
    tecla(&mut settings, "7");
    tecla(&mut settings, "l");
    assert_eq!(settings.notice, Some(Notice::Error("Perfil de energia: programa não encontrado".to_owned())));
}
