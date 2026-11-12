use super::message::Message;
use super::state::Page;
use super::tests::{app, store};
use super::update::update;
use crate::domain::updates::{
    UpdateGroup, UpdateItem, UpdateKind, UpdateReport, UpdateSource, UpdateTarget,
};
use crate::domain::{Installation, Manager};

fn update_report() -> UpdateReport {
    UpdateReport {
        groups: vec![
            UpdateGroup {
                source: UpdateSource::Flatpak(Installation::User),
                items: vec![UpdateItem {
                    id: "app/org.gimp.GIMP/x86_64/stable".into(),
                    name: "GIMP".into(),
                    installed: Some("3.0".into()),
                    available: Some("3.1".into()),
                    source: UpdateSource::Flatpak(Installation::User),
                    kind: UpdateKind::Application,
                }],
                error: None,
            },
            UpdateGroup {
                source: UpdateSource::Native(Manager::Pacman),
                items: vec![UpdateItem {
                    id: "mesa".into(),
                    name: "mesa".into(),
                    installed: Some("24.1".into()),
                    available: Some("24.2".into()),
                    source: UpdateSource::Native(Manager::Pacman),
                    kind: UpdateKind::System,
                }],
                error: None,
            },
        ],
        warning: None,
    }
}

#[test]
fn atualizacoes_nao_reutilizam_lista_de_instalados() {
    let mut state = store();
    state.apps = vec![app()];
    let _ = update(&mut state, Message::ShowUpdates);
    assert_eq!(state.page, Page::Updates);
    assert!(state.selected.is_none());
    assert!(state.busy);
    let _ = update(&mut state, Message::UpdatesChecked(Ok(update_report())));
    assert_eq!(state.updates.as_ref().map(UpdateReport::count), Some(2));
    assert!(!state.busy);
}

#[test]
fn atualiza_tudo_em_sequencia_e_reconsulta_apos_falha_parcial() {
    let mut state = store();
    state.page = Page::Updates;
    state.updates = Some(update_report());
    let _ = update(&mut state, Message::AskUpdateAll);
    assert_eq!(state.update_pending.as_ref().map(Vec::len), Some(2));
    let _ = update(&mut state, Message::ConfirmUpdate);
    assert!(matches!(state.updating, Some(UpdateTarget::Flatpak { .. })));
    assert!(state.busy);
    let _ = update(
        &mut state,
        Message::UpdateFinished(Err("flatpak falhou".into())),
    );
    assert_eq!(
        state.updating,
        Some(UpdateTarget::Native {
            manager: Manager::Pacman,
            package: None,
        })
    );
    let _ = update(&mut state, Message::UpdateFinished(Ok(())));
    assert!(state.updating.is_none());
    assert_eq!(state.error.as_deref(), Some("flatpak falhou"));
    assert!(state.busy);
    let _ = update(&mut state, Message::UpdatesChecked(Ok(update_report())));
    assert!(!state.busy);
    assert_eq!(state.error.as_deref(), Some("flatpak falhou"));
}

#[test]
fn atualizacao_individual_arch_exige_confirmacao_do_sistema() {
    let mut state = store();
    state.updates = Some(update_report());
    let target = state.updates.as_ref().unwrap().groups[1].items[0].target();
    let _ = update(&mut state, Message::AskUpdate(target));
    assert_eq!(
        state.update_pending,
        Some(vec![UpdateTarget::Native {
            manager: Manager::Pacman,
            package: None,
        }])
    );
    let _ = update(&mut state, Message::Cancel);
    assert!(state.update_pending.is_none());
}
