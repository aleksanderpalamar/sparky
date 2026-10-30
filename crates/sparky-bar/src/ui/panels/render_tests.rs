use super::{fixtures::ready, parts, style, view};
use crate::{
    app::{message::Message, panel_flow_tests::wifi, panel_state::Panels},
    domain::{
        panel::PanelKind,
        prompt::{Prompt, PromptId, PromptKind},
        secret::Secret,
    },
};
use iced::{
    Element, Pixels, Rectangle, Renderer, Size,
    advanced::{
        layout::{Layout, Limits},
        renderer::{Headless, Style},
        widget::Tree,
    },
    futures::executor::block_on,
    mouse,
};
use sparky_theme::{Palette, Skin};

fn render(skin: Skin, panels: &Panels, kind: PanelKind) -> Vec<u8> {
    let center = crate::app::center_state::CenterState::default();
    draw(skin, view(&skin, panels, &center, kind), kind.size())
}

fn draw(skin: Skin, mut view: Element<'_, Message>, (width, height): (u32, u32)) -> Vec<u8> {
    let mut renderer = block_on(<Renderer as Headless>::new(
        sparky_ui::style::FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut tree = Tree::new(&view);
    let size = Size::new(width as f32, height as f32);
    let node = view
        .as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, size));
    assert_eq!(node.size(), size);
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
    renderer.screenshot(Size::new(width, height), 1.0, iced::Color::TRANSPARENT)
}

#[test]
fn paineis_e_formularios_renderizam_sem_servidor_grafico_nos_temas_claro_e_escuro() {
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
        for kind in [
            PanelKind::Clock,
            PanelKind::Audio,
            PanelKind::Network,
            PanelKind::Bluetooth,
        ] {
            let pixels = render(skin, &ready(), kind);
            assert!(pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
            if let Ok(path) = std::env::var("SPARKY_PANEL_SNAPSHOTS") {
                save(&format!("{path}/{name}-{kind:?}.ppm"), &pixels, kind.size());
            }
        }
        let mut panels = ready();
        panels.wifi = Some(crate::app::panel_state::WifiPassword {
            network: wifi(),
            password: Secret::new("12345678".to_owned()),
            remember: true,
        });
        let _ = render(skin, &panels, PanelKind::Network);
        panels.prompt = Some(Prompt {
            id: PromptId(1),
            kind: PromptKind::Confirm {
                device: "Teclado".to_owned(),
                code: Some(1234),
            },
        });
        let _ = render(skin, &panels, PanelKind::Bluetooth);
    }
}

#[test]
fn superficie_opaca_e_controles_retangulares_seguem_o_skin() {
    let mut skin = Skin::default();
    skin.background.a = 0.2;
    let surface = style::surface(skin);
    let Some(iced::Background::Color(color)) = surface.background else {
        panic!("sem fundo")
    };
    assert_eq!(color.a, 1.0);
    assert_eq!(color.r, skin.background.r);
    assert_eq!(surface.border.radius, 0.0.into());
    assert_eq!(surface.border.color, skin.accent);
    let volume = style::volume(skin)(&skin.theme(), iced::widget::slider::Status::Active);
    assert!(matches!(
        volume.handle.shape,
        iced::widget::slider::HandleShape::Rectangle { .. }
    ));
}

#[test]
fn interruptor_tem_trilho_e_botao_deslizante_como_a_referencia() {
    let skin = Skin::new(Palette::from_colors(
        "background = \"#1a1b26\"\nforeground = \"#a9b1d6\"",
    ));
    let rgb = |color: iced::Color| color.into_rgba8()[..3].to_vec();
    let pixel = |pixels: &[u8], x: usize, y: usize| pixels[(y * 42 + x) * 4..][..3].to_vec();
    let ligado = draw(skin, parts::switch(skin, true, Message::Ignore), (42, 22));
    assert_eq!(pixel(&ligado, 0, 11), rgb(skin.foreground));
    assert_eq!(pixel(&ligado, 21, 0), rgb(skin.foreground));
    assert_eq!(pixel(&ligado, 31, 11), rgb(skin.foreground));
    assert_ne!(pixel(&ligado, 10, 11), rgb(skin.foreground));
    let desligado = draw(skin, parts::switch(skin, false, Message::Ignore), (42, 22));
    assert_eq!(pixel(&desligado, 10, 11), rgb(skin.dark_foreground));
    assert_eq!(pixel(&desligado, 31, 11), pixel(&ligado, 10, 11));
}

fn save(path: &str, pixels: &[u8], (width, height): (u32, u32)) {
    let mut bytes = format!("P6\n{width} {height}\n255\n").into_bytes();
    bytes.extend(
        pixels
            .chunks_exact(4)
            .flat_map(|pixel| pixel[..3].iter().copied()),
    );
    std::fs::write(path, bytes).unwrap();
}
