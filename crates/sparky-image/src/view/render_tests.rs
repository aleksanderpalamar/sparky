use iced::advanced::layout::{Layout, Limits};
use iced::advanced::renderer::{Headless, Style};
use iced::advanced::widget::Tree;
use iced::futures::executor::block_on;
use iced::{Pixels, Rectangle, Renderer, Size, mouse};

use crate::app::state::Viewer;
use crate::app::test_support::{JANELA, abrir, concluir};

fn render(viewer: &Viewer) -> Vec<u8> {
    let mut renderer = block_on(<Renderer as Headless>::new(
        sparky_ui::style::FONT,
        Pixels(14.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut view = super::view(viewer);
    let mut tree = Tree::new(&view);
    let node = view
        .as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, JANELA));
    view.as_widget().draw(
        &tree,
        &mut renderer,
        &viewer.skin.theme(),
        &Style {
            text_color: viewer.skin.foreground,
        },
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &Rectangle::with_size(JANELA),
    );
    let size = Size::new(JANELA.width as u32, JANELA.height as u32);
    let pixels = renderer.screenshot(size, 1.0, viewer.skin.background);
    if let Ok(path) = std::env::var("SPARKY_IMAGE_SNAPSHOT") {
        let mut bytes = format!("P6\n{} {}\n255\n", size.width, size.height).into_bytes();
        bytes.extend(pixels.chunks_exact(4).flat_map(|pixel| pixel[..3].to_vec()));
        std::fs::write(path, bytes).unwrap();
    }
    pixels
}

fn pixel(pixels: &[u8], x: usize, y: usize) -> [u8; 3] {
    let index = (y * JANELA.width as usize + x) * 4;
    [pixels[index], pixels[index + 1], pixels[index + 2]]
}

fn rgb(color: iced::Color) -> [u8; 3] {
    let [r, g, b, _] = color.into_rgba8();
    [r, g, b]
}

#[test]
fn imagem_fica_centralizada_sobre_o_fundo_com_status_embaixo() {
    let (mut viewer, task) = abrir(vec!["a.png"], Some("/fotos/a.png"));
    concluir(&mut viewer, task);
    let pixels = render(&viewer);
    assert_eq!(pixel(&pixels, 500, 400), [200, 30, 30]);
    assert_eq!(pixel(&pixels, 420, 400), [200, 30, 30]);
    assert_eq!(pixel(&pixels, 380, 400), rgb(viewer.skin.background));
    assert_eq!(pixel(&pixels, 500, 320), rgb(viewer.skin.background));
    assert_eq!(pixel(&pixels, 2, 830), rgb(viewer.skin.lighter_background));
}

#[test]
fn imagem_girada_ocupa_a_largura_e_altura_trocadas() {
    let (mut viewer, task) = abrir(vec!["a.png"], Some("/fotos/a.png"));
    concluir(&mut viewer, task);
    let task = crate::app::update::update(
        &mut viewer,
        crate::app::message::Message::Command(crate::app::keys::Command::Rotate(
            crate::domain::turn::Spin::Clockwise,
        )),
    );
    concluir(&mut viewer, task);
    let pixels = render(&viewer);
    assert_eq!(pixel(&pixels, 500, 320), [200, 30, 30]);
    assert_eq!(pixel(&pixels, 420, 400), rgb(viewer.skin.background));
}
