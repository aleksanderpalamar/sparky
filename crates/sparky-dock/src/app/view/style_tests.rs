use iced::border::Radius;
use iced::widget::{button, checkbox};
use iced::{Background, Color};
use sparky_theme::{Palette, Skin};

use super::style::{
    add, badge, card, check, dialog, divider, mark, menu, primary, secondary, strip, tray,
};
use crate::domain::presence::Presence;

fn reto() -> Radius {
    Radius::from(0.0)
}

fn color(background: Option<Background>) -> Option<Color> {
    match background {
        Some(Background::Color(color)) => Some(color),
        _ => None,
    }
}

#[test]
fn nenhuma_superficie_do_dock_tem_canto_arredondado() {
    let skin = Skin::default();
    assert_eq!(tray(skin, 0.0).border.radius, reto());
    assert_eq!(dialog(skin).border.radius, reto());
    assert_eq!(card(skin).border.radius, reto());
    assert_eq!(divider(skin).border.radius, reto());
    assert_eq!(badge(skin).border.radius, reto());
    assert_eq!(menu(skin, 0.0).border.radius, reto());
    assert_eq!(strip(skin, 0.0).border.radius, reto());
    for status in [
        button::Status::Active,
        button::Status::Hovered,
        button::Status::Disabled,
    ] {
        assert_eq!(primary(skin, status).border.radius, reto());
        assert_eq!(secondary(skin, status).border.radius, reto());
        assert_eq!(add(skin, status).border.radius, reto());
    }
    let marcada = checkbox::Status::Active { is_checked: true };
    assert_eq!(check(skin, marcada).border.radius, reto());
}

#[test]
fn as_cores_seguem_o_tema_ativo() {
    let skin = Skin::new(Palette {
        accent: sparky_theme::Rgb::new(200, 40, 40),
        ..Palette::default()
    });
    assert_eq!(dialog(skin).border.color, skin.accent);
    assert_eq!(mark(skin, Presence::Focused), Some(skin.accent));
    let fundo = color(tray(skin, 0.0).background);
    assert_eq!(
        fundo.map(|cor| (cor.r, cor.g, cor.b)),
        Some((
            skin.lighter_background.r,
            skin.lighter_background.g,
            skin.lighter_background.b
        ))
    );
    assert_eq!(fundo.map(|cor| cor.a), Some(1.0));
    let sem_quadrado = color(add(skin, button::Status::Active).background);
    assert_eq!(sem_quadrado.map(|cor| cor.a), Some(0.0));
    let principal = color(primary(skin, button::Status::Active).background);
    assert_eq!(principal, Some(skin.accent));
    let marcada = check(skin, checkbox::Status::Active { is_checked: true });
    assert_eq!(color(Some(marcada.background)), Some(skin.accent));
}

#[test]
fn so_aplicativo_aberto_ganha_indicador() {
    let skin = Skin::default();
    assert_eq!(mark(skin, Presence::Closed), None);
    assert_eq!(mark(skin, Presence::Running), Some(skin.muted));
}

#[test]
fn com_cantos_arredondados_o_fundo_o_menu_e_a_faixa_usam_o_raio_escolhido() {
    let skin = Skin::default();
    let raio = Radius::from(3.0);
    assert_eq!(tray(skin, 3.0).border.radius, raio);
    assert_eq!(menu(skin, 3.0).border.radius, raio);
    assert_eq!(strip(skin, 3.0).border.radius, raio);
    assert_eq!(dialog(skin).border.radius, reto());
}

#[test]
fn com_frosted_glass_o_fundo_e_o_menu_seguem_a_transparencia_do_tema() {
    let vidro = Skin::new(Palette {
        glass: true,
        ..Palette::default()
    });
    let fundo = color(tray(vidro, 0.0).background);
    assert_eq!(fundo.map(|cor| cor.a), Some(vidro.background.a));
    assert_eq!(fundo.map(|cor| cor.r), Some(vidro.lighter_background.r));
    let menu_fundo = color(menu(vidro, 0.0).background);
    assert_eq!(menu_fundo.map(|cor| cor.a), Some(vidro.background.a));
    let dialogo = color(dialog(vidro).background);
    assert_eq!(dialogo.map(|cor| cor.a), Some(1.0));
}
