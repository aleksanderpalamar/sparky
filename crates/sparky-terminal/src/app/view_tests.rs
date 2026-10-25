use iced::advanced::layout::Limits;
use iced::advanced::renderer::Headless;
use iced::advanced::widget::Tree;
use iced::futures::executor::block_on;
use iced::{Pixels, Renderer, Size};
use sparky_ui::style::FONT;

use crate::app::state::{HEIGHT, WIDTH};
use crate::app::state_tests::terminal;
use crate::app::view::view;

#[test]
fn a_tela_do_terminal_ocupa_a_janela_inteira() {
    let Some(renderer) = block_on(<Renderer as Headless>::new(
        FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    )) else {
        panic!("o renderizador de CPU não abriu")
    };
    let aberto = terminal();
    let mut tela = view(crate::domain::tabs::TabId(0), &aberto);
    let mut arvore = Tree::new(&tela);
    let janela = Size::new(WIDTH, HEIGHT);
    let layout =
        tela.as_widget_mut()
            .layout(&mut arvore, &renderer, &Limits::new(Size::ZERO, janela));
    let pintura = layout
        .children()
        .first()
        .map(|camada| camada.bounds().size());
    assert_eq!(pintura, Some(janela));
}

#[test]
fn varias_abas_cabem_na_barra_sem_cobrir_o_terminal() {
    let Some(renderer) = block_on(<Renderer as Headless>::new(
        FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    )) else {
        panic!("sem renderizador de CPU")
    };
    let mut app = super::workspace_tests::workspace();
    for _ in 0..12 {
        app.new_tab();
    }
    let mut view = super::tabs_view::view(&app);
    let mut tree = Tree::new(&view);
    let size = Size::new(WIDTH, HEIGHT);
    let layout = view
        .as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, size));
    assert_eq!(layout.bounds().size(), size);
    let children = layout.children();
    assert_eq!(children[0].bounds().height, super::workspace::TAB_HEIGHT);
    assert_eq!(children[1].bounds().y, super::workspace::TAB_HEIGHT);
    assert_eq!(
        children[1].bounds().height,
        HEIGHT - super::workspace::TAB_HEIGHT
    );
}

#[test]
fn titulos_longos_nao_aumentam_a_barra_nem_perdem_unicode() {
    let title = super::tabs_view::caption(12, "área\nterminal com um título muito comprido");
    assert!(title.starts_with("13: área"));
    assert!(!title.chars().any(char::is_control));
    assert!(title.chars().count() <= 17);
}
