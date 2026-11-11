use std::sync::Arc;

use super::message::Message;
use super::state::Store;
use super::update::update;
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
        Ok(SearchResult {
            apps: Vec::new(),
            source: None,
            warning: None,
            flathub_missing: false,
        })
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

fn store() -> Store {
    Store::new(Arc::new(Fake), sparky_theme::Skin::default())
}

fn app() -> Application {
    Application {
        id: "org.gimp.GIMP".into(),
        name: "GIMP".into(),
        summary: "Editor".into(),
        version: Some("3.0".into()),
        source: Source::Flathub(Installation::System),
        installed: false,
    }
}

#[test]
fn busca_prioriza_flathub_e_selecao_exige_confirmacao() {
    let mut state = store();
    let _ = update(&mut state, Message::Query("gimp".into()));
    let _ = update(&mut state, Message::Search);
    assert!(state.busy);
    let _ = update(
        &mut state,
        Message::Searched(Ok(SearchResult {
            apps: vec![app()],
            source: Some(Source::Flathub(Installation::System)),
            warning: None,
            flathub_missing: false,
        })),
    );
    let _ = update(&mut state, Message::Select(0));
    let _ = update(&mut state, Message::Ask(Action::Install));
    assert_eq!(
        state.pending.as_ref().map(|(_, action)| *action),
        Some(Action::Install)
    );
    assert!(!state.busy);
    let _ = update(&mut state, Message::Cancel);
    assert!(state.pending.is_none());
}

#[test]
fn erro_de_operacao_fica_visivel_e_paleta_muda() {
    let mut state = store();
    let palette = sparky_theme::Palette::from_colors("accent = \"#123456\"");
    let _ = update(&mut state, Message::Palette(palette));
    assert_eq!(state.skin, sparky_theme::Skin::new(palette));
    let _ = update(&mut state, Message::Performed(Err("falhou".into())));
    assert_eq!(state.error.as_deref(), Some("falhou"));
}

#[test]
fn troca_de_selecao_cancela_confirmacao_antiga() {
    let mut state = store();
    state.apps = vec![app(), app()];
    let _ = update(&mut state, Message::Select(0));
    let _ = update(&mut state, Message::Ask(Action::Install));
    let _ = update(&mut state, Message::Select(1));
    assert!(state.pending.is_none());
}

#[test]
fn sucesso_permanece_visivel_durante_recarga() {
    let mut state = store();
    let _ = update(&mut state, Message::Performed(Ok(())));
    assert!(state.busy);
    assert!(state.notice.is_some());
    let _ = update(
        &mut state,
        Message::Browsed(Ok(SearchResult {
            apps: Vec::new(),
            source: None,
            warning: None,
            flathub_missing: false,
        })),
    );
    assert!(state.notice.is_some());
}
