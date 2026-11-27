use super::context::{Row, Target, entries};

fn linhas(target: Target, has_clip: bool, has_entries: bool) -> Vec<String> {
    entries(target, has_clip, has_entries)
        .into_iter()
        .map(|row| match row {
            Row::Item {
                label,
                shortcut,
                enabled,
                ..
            } => format!("{label}|{shortcut}|{}", if enabled { "on" } else { "off" }),
            Row::Palette => "paleta".into(),
            Row::Separator => "---".into(),
        })
        .collect()
}

fn arquivo(extrair: bool, email: &str) -> Vec<String> {
    let mut rows = vec![
        "menu-open|Enter|on",
        "menu-open-with||on",
        "---",
        "menu-cut|Ctrl+X|on",
        "menu-copy|Ctrl+C|on",
        "menu-move-to||on",
        "menu-copy-to||on",
        "---",
        "menu-rename|F2|on",
        "menu-compress||on",
    ];
    if extrair {
        rows.push("menu-extract|e|on");
    }
    rows.extend([
        email,
        "menu-trash|Delete|on",
        "menu-delete|Shift+Delete|on",
        "---",
        "menu-properties|Alt+Enter|on",
    ]);
    rows.into_iter().map(String::from).collect()
}

fn fundo(colar: &str, selecionar: &str) -> Vec<String> {
    [
        "menu-new-folder|Ctrl+Shift+N|on",
        "menu-new-file||on",
        "menu-open-with||on",
        "---",
        colar,
        selecionar,
        "---",
        "menu-properties|Alt+Enter|on",
    ]
    .map(String::from)
    .to_vec()
}

#[test]
fn menu_de_arquivo_mantem_ordem_atalhos_e_estado() {
    for (clip, itens) in [(false, true), (true, false)] {
        assert_eq!(
            linhas(Target::File, clip, itens),
            arquivo(false, "menu-email||on")
        );
    }
}

#[test]
fn menu_de_compactado_acrescenta_extrair_antes_do_email() {
    assert_eq!(
        linhas(Target::Archive, false, true),
        arquivo(true, "menu-email||on")
    );
}

#[test]
fn menu_de_pasta_mantem_email_desabilitado() {
    assert_eq!(
        linhas(Target::Folder, false, true),
        arquivo(false, "menu-email||off")
    );
}

#[test]
fn menu_do_fundo_depende_do_clipboard_e_dos_itens() {
    let casos = [
        (
            false,
            false,
            "menu-paste|Ctrl+V|off",
            "menu-select-all|Ctrl+A|off",
        ),
        (
            true,
            false,
            "menu-paste|Ctrl+V|on",
            "menu-select-all|Ctrl+A|off",
        ),
        (
            false,
            true,
            "menu-paste|Ctrl+V|off",
            "menu-select-all|Ctrl+A|on",
        ),
        (
            true,
            true,
            "menu-paste|Ctrl+V|on",
            "menu-select-all|Ctrl+A|on",
        ),
    ];
    for (clip, itens, colar, selecionar) in casos {
        assert_eq!(
            linhas(Target::Background, clip, itens),
            fundo(colar, selecionar)
        );
    }
}

#[test]
fn menus_da_lixeira_mantem_apenas_restauracao() {
    assert_eq!(
        linhas(Target::Trashed, true, true),
        ["menu-restore||on", "menu-properties|Alt+Enter|on"]
    );
    assert_eq!(
        linhas(Target::TrashBackground, false, true),
        ["menu-restore-all||on"]
    );
    assert_eq!(
        linhas(Target::TrashBackground, true, false),
        ["menu-restore-all||off"]
    );
}

#[test]
fn menu_do_marcador_mantem_paleta_sem_cor_e_remover() {
    assert_eq!(
        linhas(Target::Bookmark(3), true, false),
        [
            "paleta",
            "menu-no-color||on",
            "---",
            "menu-remove-bookmark||on"
        ]
    );
}
