use std::sync::Arc;

use iced::advanced::layout::{Layout, Limits};
use iced::advanced::renderer::{Headless, Style};
use iced::advanced::widget::{Id, Operation, Tree};
use iced::futures::executor::block_on;
use iced::{Pixels, Rectangle, Renderer, Size, mouse};

use crate::app::{Page, Store};
use crate::domain::{Action, Application, Installation, SearchResult, SearchSource, Source};
use crate::ports::StoreBackend;

#[derive(Debug)]
struct Fake;

impl StoreBackend for Fake {
    fn browse(&self) -> Result<SearchResult, String> {
        Ok(SearchResult {
            apps: Vec::new(),
            source: None,
            warning: None,
            flathub_missing: false,
        })
    }

    fn search(&self, _: &str, _: SearchSource) -> Result<SearchResult, String> {
        self.browse()
    }

    fn installed(&self) -> Result<Vec<Application>, String> {
        Ok(Vec::new())
    }

    fn perform(&self, _: &Application, _: Action) -> Result<(), String> {
        Ok(())
    }

    fn enable_flathub(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Default)]
struct Texts(Vec<String>);

impl Operation for Texts {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn text(&mut self, _: Option<&Id>, _: Rectangle, value: &str) {
        self.0.push(value.to_owned());
    }
}

fn store() -> Store {
    let mut store = Store::new(Arc::new(Fake), sparky_theme::Skin::default());
    store.apps = vec![Application {
        id: "org.example.Paint".into(),
        name: "Paint".into(),
        summary: "Editor de imagens".into(),
        version: Some("1.0".into()),
        source: Source::Flathub(Installation::System),
        installed: true,
    }];
    store
}

fn render(store: &Store, size: Size, name: &str) -> Vec<String> {
    let mut renderer = block_on(<Renderer as Headless>::new(
        sparky_ui::style::FONT,
        Pixels(14.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut view = super::view(store);
    let mut tree = Tree::new(&view);
    let node = view
        .as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, size));
    view.as_widget().draw(
        &tree,
        &mut renderer,
        &store.skin.theme(),
        &Style {
            text_color: store.skin.foreground,
        },
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &Rectangle::with_size(size),
    );
    let dimensions = Size::new(size.width as u32, size.height as u32);
    if let Ok(folder) = std::env::var("SPARKY_STORE_SNAPSHOTS") {
        let pixels = renderer.screenshot(dimensions, 1.0, store.skin.background);
        let mut data =
            format!("P6\n{} {}\n255\n", dimensions.width, dimensions.height).into_bytes();
        data.extend(pixels.chunks_exact(4).flat_map(|pixel| pixel[..3].to_vec()));
        std::fs::write(format!("{folder}/{name}.ppm"), data).unwrap();
    }
    let mut texts = Texts::default();
    view.as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut texts);
    texts.0
}

#[test]
fn explorar_e_detalhe_renderizam_em_janelas_larga_e_estreita() {
    let mut state = store();
    for size in [Size::new(1050.0, 680.0), Size::new(650.0, 600.0)] {
        let texts = render(&state, size, "explorar");
        assert!(texts.contains(&"Conheça aplicativos do catálogo".to_owned()));
        assert!(texts.contains(&"Paint".to_owned()));
    }
    state.selected = Some(0);
    let texts = render(&state, Size::new(1050.0, 680.0), "detalhe");
    assert!(texts.contains(&"Prévia indisponível para este aplicativo".to_owned()));
    assert!(texts.contains(&"Remover".to_owned()));
}

#[test]
fn instalados_e_atualizacoes_nao_inventam_pacotes_pendentes() {
    let mut state = store();
    state.page = Page::Installed;
    let texts = render(&state, Size::new(1050.0, 680.0), "instalados");
    assert!(texts.contains(&"Paint".to_owned()));
    state.page = Page::Updates;
    let texts = render(&state, Size::new(1050.0, 680.0), "atualizacoes");
    assert!(
        texts
            .iter()
            .any(|value| value.starts_with("A consulta de atualizações"))
    );
    assert!(!texts.contains(&"Paint".to_owned()));
}
