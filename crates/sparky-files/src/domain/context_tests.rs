use super::context::{Action, Target, entries, position};

#[test]
fn arquivo_e_pasta_expoem_operacoes_reais_e_propriedades() {
    for target in [Target::File, Target::Folder] {
        let actions: Vec<_> = entries(target, false, true)
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
            Action::Properties,
        ] {
            assert!(actions.contains(&expected), "{target:?}: {expected:?}");
        }
    }
    assert!(
        entries(Target::Folder, false, true)
            .iter()
            .any(|row| row.is_disabled(Action::Email))
    );
}

#[test]
fn area_vazia_oferece_criacao_e_so_habilita_colar_com_clipboard() {
    let rows = entries(Target::Background, false, true);
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
        entries(Target::Background, true, false)
            .iter()
            .any(|row| row.is_disabled(Action::SelectAll))
    );
}

#[test]
fn menu_permanece_dentro_da_janela_junto_das_bordas() {
    assert_eq!(
        position((950.0, 700.0), (1000.0, 760.0), (280.0, 460.0)),
        (720.0, 300.0)
    );
    assert_eq!(
        position((20.0, 30.0), (1000.0, 760.0), (280.0, 460.0)),
        (20.0, 30.0)
    );
    assert_eq!(
        position((20.0, 30.0), (100.0, 80.0), (280.0, 460.0)),
        (0.0, 0.0)
    );
}
