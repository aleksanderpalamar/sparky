use iced::advanced::Layout;
use iced::advanced::layout::Limits;
use iced::advanced::renderer::Headless;
use iced::advanced::widget::{Id, Operation, Tree};
use iced::futures::executor::block_on;
use iced::{Pixels, Rectangle, Renderer, Size};
use sparky_ui::style::FONT;

use super::state::Settings;
use super::test_support::{Sparky, painel};
use crate::domain::hardware::Hardware;

#[derive(Default)]
struct Texts(Vec<String>);

impl Operation for Texts {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn text(&mut self, _: Option<&Id>, _: Rectangle, text: &str) {
        self.0.push(text.to_owned());
    }
}

fn texts(settings: &Settings) -> Vec<String> {
    let renderer = block_on(<Renderer as Headless>::new(
        FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut view = crate::view::about::view(settings);
    let mut tree = Tree::new(&view);
    let node = view.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, Size::new(700.0, 500.0)),
    );
    let mut texts = Texts::default();
    view.as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut texts);
    texts.0
}

#[test]
fn sobre_mostra_hardware_em_vez_do_caminho_da_configuracao() {
    let (settings, _) = painel(None, Sparky::default());
    let visible = texts(&settings);
    assert!(
        !visible
            .iter()
            .any(|text| text == "Configuração" || text.contains("sparky.toml"))
    );
    for label in ["Processador", "GPU", "RAM"] {
        assert!(visible.iter().any(|text| text == label), "{label}");
    }
}

#[test]
fn modelos_de_hardware_e_ram_total_aparecem_na_tela() {
    let (mut settings, _) = painel(None, Sparky::default());
    settings.context.about.hardware = Hardware {
        processor: Some("AMD Ryzen 5 5500".into()),
        gpus: vec!["AMD Radeon RX 6600".into(), "Intel UHD Graphics".into()],
        memory_kib: Some(16_150_580),
    };
    let visible = texts(&settings);
    for value in [
        "AMD Ryzen 5 5500",
        "AMD Radeon RX 6600\nIntel UHD Graphics",
        "15,4 GiB",
    ] {
        assert!(visible.iter().any(|text| text == value), "{value}");
    }
}

#[test]
fn informacoes_ausentes_ficam_explicitas_na_tela() {
    let (settings, _) = painel(None, Sparky::default());
    let visible = texts(&settings);
    for label in ["Processador", "GPU", "RAM"] {
        assert!(
            visible
                .windows(2)
                .any(|pair| pair == [label, "não informado"]),
            "{label}"
        );
    }
}
