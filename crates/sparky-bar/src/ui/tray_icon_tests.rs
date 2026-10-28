use iced::advanced::{
    layout::{Layout, Limits},
    renderer::{Headless, Style},
    widget::Tree,
};
use iced::futures::executor::block_on;
use iced::{Pixels, Rectangle, Renderer, Size, mouse};
use sparky_theme::{Palette, Skin};

use super::tray_icon::icon;
use crate::domain::indicator::{Role, Status};
use crate::domain::tray::{PrimaryAction, TrayIcon, TrayItem};

#[test]
fn simbolicos_sao_renderizados_na_cor_do_tema_em_vez_da_cor_do_svg() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("test-symbolic.svg");
    std::fs::write(&path, "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\"><rect width=\"16\" height=\"16\" fill=\"#ff0000\"/></svg>").unwrap();
    for colors in [
        "background = \"#000000\"\nforeground = \"#ffffff\"",
        "background = \"#ffffff\"\nforeground = \"#222222\"",
    ] {
        let skin = Skin::new(Palette::from_colors(colors));
        let item = TrayItem {
            address: "test".to_owned(),
            title: "Teste".to_owned(),
            icon: TrayIcon::Symbolic(path.clone()),
            role: Role::Network,
            status: Status::Active,
            primary_action: PrimaryAction::Activate,
            menu_path: None,
            menu: Vec::new(),
        };
        let mut renderer = block_on(<Renderer as Headless>::new(
            sparky_ui::style::FONT,
            Pixels(12.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut view = icon(&skin, &item);
        let mut tree = Tree::new(&view);
        let size = Size::new(26.0, 26.0);
        let node =
            view.as_widget_mut()
                .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, size));
        view.as_widget().draw(
            &tree,
            &mut renderer,
            &skin.theme(),
            &Style {
                text_color: skin.foreground,
            },
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &Rectangle::with_size(size),
        );
        let pixels = renderer.screenshot(Size::new(26, 26), 1.0, skin.background);
        let middle = (13 * 26 + 13) * 4;
        let actual = &pixels[middle..middle + 4];
        let expected = skin.foreground.into_rgba8();
        assert!(
            actual.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 1),
            "{actual:?} != {expected:?}"
        );
    }
}
