use iced::Background;
use sparky_theme::{Palette, Skin};

use crate::look::floating;

fn fundo(skin: Skin) -> Option<iced::Color> {
    match floating(skin).background {
        Some(Background::Color(color)) => Some(color),
        _ => None,
    }
}

#[test]
fn menu_em_popup_proprio_segue_o_vidro_da_janela() {
    let vidro = Skin::new(Palette {
        glass: true,
        ..Palette::SPARKY
    });
    let cor = fundo(vidro);
    assert_eq!(cor.map(|cor| cor.a), Some(vidro.background.a));
    assert_eq!(cor.map(|cor| cor.r), Some(vidro.raised.r));
}

#[test]
fn sem_vidro_o_menu_e_opaco() {
    let opaco = Skin::new(Palette::SPARKY);
    assert_eq!(fundo(opaco).map(|cor| cor.a), Some(1.0));
}
