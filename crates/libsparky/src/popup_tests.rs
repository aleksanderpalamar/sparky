use iced::Point;
use iced_layershell::reexport::PopupGravity;

use crate::popup::{Step, at, step};

#[test]
fn o_menu_abre_no_ponto_do_clique_para_baixo_e_a_direita() {
    let janela = iced::window::Id::unique();
    let menu = at(janela, Point::new(120.4, 80.6), (280.0, 300.0));
    assert_eq!(menu.parent, Some(janela));
    assert_eq!(menu.size, (280, 300));
    assert_eq!(
        menu.placement,
        iced_layershell::reexport::PopupPlacement::Position((120, 81))
    );
    assert_eq!(menu.gravity, PopupGravity::BottomRight);
}

#[test]
fn menu_nunca_tem_tamanho_zero() {
    let menu = at(iced::window::Id::unique(), Point::ORIGIN, (0.0, -5.0));
    assert_eq!(menu.size, (1, 1));
}

#[test]
fn o_menu_abre_quando_pedido_e_fecha_quando_some_o_pedido() {
    let aberto = iced::window::Id::unique();
    assert_eq!(step(true, None), Step::Open);
    assert_eq!(step(true, Some(aberto)), Step::Keep);
    assert_eq!(step(false, Some(aberto)), Step::Close(aberto));
    assert_eq!(step(false, None), Step::Keep);
}
