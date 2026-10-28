use super::popup::{self, Geometry};
use iced_layershell::reexport::KeyboardInteractivity;

#[test]
fn popup_nao_toma_foco_e_cabe_no_monitor() {
    assert!(matches!(
        popup::settings((360, 200), None).keyboard_interactivity,
        KeyboardInteractivity::None
    ));
    assert_eq!(
        popup::size(
            3,
            &Geometry {
                width: 320,
                height: 240,
                output: None,
            }
        ),
        (300, 182)
    );
    assert_eq!(popup::size(10, &Geometry::default()), (360, 600));
}

#[test]
fn popups_renderizam_sem_display_e_acompanham_paletas() {
    use crate::domain::{Center, Incoming};
    use iced::{
        Pixels, Rectangle, Renderer, Size,
        advanced::{
            layout::{Layout, Limits},
            renderer::{Headless, Style},
            widget::Tree,
        },
        futures::executor::block_on,
        mouse,
    };
    use sparky_theme::{Palette, Skin};
    let mut center = Center::default();
    center
        .notify(
            Incoming {
                app: "Sparky".to_owned(),
                title: "Captura concluída".to_owned(),
                body: "Vídeo salvo com sucesso".to_owned(),
                timeout: 5000,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    let cards = center.popups();
    for (name, colors) in [
        (
            "escuro",
            "background = \"#17232d\"\nforeground = \"#d0d8df\"",
        ),
        (
            "claro",
            "background = \"#efefef\"\nforeground = \"#222222\"",
        ),
    ] {
        let skin = Skin::new(Palette::from_colors(colors));
        let mut renderer = block_on(<Renderer as Headless>::new(
            sparky_ui::style::FONT,
            Pixels(12.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut view = popup::view(&skin, &cards);
        let mut tree = Tree::new(&view);
        let size = Size::new(360.0, 200.0);
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
        let pixels = renderer.screenshot(Size::new(360, 200), 1.0, iced::Color::TRANSPARENT);
        assert!(pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
        if let Ok(directory) = std::env::var("SPARKY_PANEL_SNAPSHOTS") {
            let mut bytes = b"P6\n360 200\n255\n".to_vec();
            bytes.extend(
                pixels
                    .chunks_exact(4)
                    .flat_map(|pixel| pixel[..3].iter().copied()),
            );
            std::fs::write(format!("{directory}/popup-{name}.ppm"), bytes).unwrap();
        }
    }
}

#[test]
fn icone_nao_consume_a_largura_destinada_ao_texto() {
    use crate::domain::{Center, Incoming};
    use iced::{
        Pixels, Renderer, Size,
        advanced::{layout::Limits, renderer::Headless, widget::Tree},
        futures::executor::block_on,
    };
    let mut center = Center::default();
    center.notify(Incoming::default(), 0).unwrap();
    let cards = center.popups();
    let renderer = block_on(<Renderer as Headless>::new(
        sparky_ui::style::FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let skin = sparky_theme::Skin::default();
    let mut view = crate::presentation::card(
        &skin,
        &cards[0],
        crate::presentation::Density::Popup,
        |_, _| (),
        (),
    );
    let mut tree = Tree::new(&view);
    let node = view.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, Size::new(360.0, 200.0)),
    );
    let icon = &node.children()[0].children()[1].children()[0];
    assert_eq!(icon.size(), Size::new(40.0, 40.0));
}
