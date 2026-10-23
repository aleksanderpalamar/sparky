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
    let mut tela = view(&aberto);
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
