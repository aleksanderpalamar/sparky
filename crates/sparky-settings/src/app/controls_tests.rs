use std::rc::Rc;

use crate::app::message::Message;
use crate::app::state::Target;
use crate::app::test_support::{Comandos, Sparky, painel, painel_com, salvo};
use crate::app::update::update;
use crate::domain::field::Field;
use crate::domain::section::Section;
use crate::domain::system::RowId;
use crate::ports::fake::FakeCommands;

const BRILHO: &str = "brightnessctl -d intel_backlight set 70%";
const IDIOMA: &str = "localectl set-locale LANG=C.UTF-8";

fn sistema() -> Comandos {
    Comandos(Rc::new(
        FakeCommands::new()
            .with(
                "brightnessctl -m -c backlight -l",
                "intel_backlight,backlight,600,45%,1200\n",
            )
            .with(
                "localectl status",
                "System Locale: LANG=pt_BR.UTF-8\nVC Keymap: br-abnt2\nX11 Layout: br\n",
            )
            .with("localectl list-locales", "C.UTF-8\npt_BR.UTF-8\n")
            .accepting(BRILHO)
            .accepting(IDIOMA),
    ))
}

#[test]
fn arrastar_so_grava_o_numero_ao_soltar() {
    let (mut settings, arquivo) = painel(None, Sparky::default());
    let alvo = Target::Field(Field::GapsInner);
    let _ = update(&mut settings, Message::Slide(alvo, 9));
    let _ = update(&mut settings, Message::Slide(alvo, 12));
    assert_eq!(settings.sliding, Some((alvo, 12)));
    assert!(!salvo(&arquivo).contains("12"));
    let _ = update(&mut settings, Message::Release);
    assert_eq!(settings.sliding, None);
    assert!(salvo(&arquivo).contains("12"));
    let _ = update(&mut settings, Message::Release);
}

#[test]
fn brilho_arrastado_e_aplicado_uma_vez_ao_soltar() {
    let commands = sistema();
    let (mut settings, _) = painel_com(None, Sparky::default(), commands.clone());
    let _ = update(&mut settings, Message::Section(Section::Power));
    let alvo = Target::System(RowId::Brightness);
    let _ = update(&mut settings, Message::Slide(alvo, 60));
    let _ = update(&mut settings, Message::Slide(alvo, 70));
    assert!(
        commands
            .0
            .calls()
            .iter()
            .all(|call| !call.contains(" set "))
    );
    let _ = update(&mut settings, Message::Release);
    let aplicados = commands
        .0
        .calls()
        .iter()
        .filter(|call| *call == BRILHO)
        .count();
    assert_eq!(aplicados, 1);
}

#[test]
fn listas_trazem_as_opcoes_e_aplicam_a_escolha() {
    let commands = sistema();
    let (mut settings, _) = painel_com(None, Sparky::default(), commands.clone());
    let opcoes = settings
        .pickers
        .get(RowId::Locale)
        .map(|state| state.options().to_vec());
    assert_eq!(
        opcoes,
        Some(vec!["C.UTF-8".to_owned(), "pt_BR.UTF-8".to_owned()])
    );
    let _ = update(&mut settings, Message::Section(Section::Region));
    let _ = update(
        &mut settings,
        Message::Choose(RowId::Locale, "C.UTF-8".to_owned()),
    );
    assert!(commands.0.calls().contains(&IDIOMA.to_owned()));
}
