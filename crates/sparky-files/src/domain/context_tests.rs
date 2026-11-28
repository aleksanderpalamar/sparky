use super::context::{Action, Offer, Target, entries};

fn offer(clip: bool, entries: bool) -> Offer {
    Offer {
        clip,
        entries,
        ..Offer::default()
    }
}

#[test]
fn arquivo_e_pasta_expoem_operacoes_reais_e_propriedades() {
    for target in [Target::File, Target::Folder] {
        let actions: Vec<_> = entries(target, offer(false, true))
            .into_iter()
            .filter_map(|row| row.action())
            .collect();
        for expected in [
            Action::Open,
            Action::OpenWith,
            Action::Cut,
            Action::Copy,
            Action::MoveTo,
            Action::CopyTo,
            Action::Rename,
            Action::Compress,
            Action::Email,
            Action::Trash,
            Action::Delete,
            Action::Properties,
        ] {
            assert!(actions.contains(&expected), "{target:?}: {expected:?}");
        }
    }
    assert!(
        entries(Target::Folder, offer(false, true))
            .iter()
            .any(|row| row.is_disabled(Action::Email))
    );
}

#[test]
fn area_vazia_oferece_criacao_e_so_habilita_colar_com_clipboard() {
    let rows = entries(Target::Background, offer(false, true));
    assert!(
        rows.iter()
            .any(|row| row.action() == Some(Action::NewFolder))
    );
    assert!(rows.iter().any(|row| row.action() == Some(Action::NewFile)));
    assert!(
        rows.iter()
            .any(|row| row.action() == Some(Action::SelectAll))
    );
    assert!(!rows.iter().any(|row| row.action() == Some(Action::Trash)));
    assert!(rows.iter().any(|row| row.is_disabled(Action::Paste)));
    assert!(
        entries(Target::Background, offer(true, false))
            .iter()
            .any(|row| row.is_disabled(Action::SelectAll))
    );
}

#[test]
fn lixeira_oferece_apenas_restauracao_e_propriedades() {
    let item: Vec<_> = entries(Target::Trashed, offer(true, true))
        .into_iter()
        .filter_map(|row| row.action())
        .collect();
    assert_eq!(item, [Action::Restore, Action::Properties]);
    let empty = entries(Target::TrashBackground, offer(true, false));
    assert!(empty.iter().any(|row| row.is_disabled(Action::RestoreAll)));
    let full = entries(Target::TrashBackground, offer(false, true));
    assert!(
        full.iter()
            .any(|row| row.action() == Some(Action::RestoreAll))
    );
}

#[test]
fn compactado_e_um_arquivo_com_extrair_aqui() {
    let arquivo: Vec<_> = entries(Target::File, offer(false, true))
        .into_iter()
        .filter_map(|row| row.action())
        .collect();
    let compactado: Vec<_> = entries(Target::Archive, offer(false, true))
        .into_iter()
        .filter_map(|row| row.action())
        .collect();
    assert!(!arquivo.contains(&Action::Extract));
    assert!(compactado.contains(&Action::Extract));
    assert!(arquivo.iter().all(|action| compactado.contains(action)));
    assert!(
        !entries(Target::Archive, offer(false, true))
            .iter()
            .any(|row| row.is_disabled(Action::Email))
    );
}

#[test]
fn o_marcador_oferece_a_paleta_sem_cor_e_remover() {
    let rows = entries(Target::Bookmark(0), offer(false, true));
    assert_eq!(rows[0], super::context::Row::Palette);
    let actions: Vec<_> = rows.iter().filter_map(|row| row.action()).collect();
    assert_eq!(actions, [Action::Tag(None), Action::RemoveBookmark]);
    let vermelho = Action::Tag(Some(super::tag_color::TagColor::Red));
    assert!(rows.iter().any(|row| row.offers(vermelho)));
    assert!(!rows.iter().any(|row| row.offers(Action::Trash)));
}

#[test]
fn a_paleta_nao_aparece_nos_menus_de_arquivos() {
    let rows = entries(Target::Folder, offer(false, true));
    let vermelho = Action::Tag(Some(super::tag_color::TagColor::Red));
    assert!(!rows.iter().any(|row| row.offers(vermelho)));
    assert!(!rows.iter().any(|row| row.offers(Action::RemoveBookmark)));
}

#[test]
fn area_vazia_e_lixeira_oferecem_voltar_e_avancar_conforme_o_historico() {
    for target in [Target::Background, Target::TrashBackground] {
        let sem = entries(target, offer(false, true));
        assert!(sem.iter().any(|row| row.is_disabled(Action::Back)));
        assert!(sem.iter().any(|row| row.is_disabled(Action::Forward)));
        let com = entries(
            target,
            Offer {
                back: true,
                forward: true,
                ..offer(false, true)
            },
        );
        assert!(com.iter().any(|row| row.offers(Action::Back)));
        assert!(com.iter().any(|row| row.offers(Action::Forward)));
    }
    for target in [Target::File, Target::Folder, Target::Trashed] {
        let rows = entries(target, offer(false, true));
        assert!(!rows.iter().any(|row| row.action() == Some(Action::Back)));
    }
}
