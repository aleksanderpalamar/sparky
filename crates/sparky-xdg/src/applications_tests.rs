use std::fs;
use std::path::{Path, PathBuf};

use super::applications::{desktop_id, directories, entries_in};

#[test]
fn a_pasta_do_usuario_vem_antes_das_do_sistema() {
    assert_eq!(
        directories(None, "/home/p", None),
        vec![
            PathBuf::from("/home/p/.local/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/usr/share/applications"),
        ]
    );
    assert_eq!(
        directories(Some("/d"), "/h", Some("/a:/b"))[1],
        PathBuf::from("/a/applications")
    );
}

#[test]
fn o_id_troca_barra_por_hifen_e_ignora_o_que_nao_e_desktop() {
    assert_eq!(
        desktop_id(Path::new("kde/org.kde.dolphin.desktop")).as_deref(),
        Some("kde-org.kde.dolphin")
    );
    assert_eq!(
        desktop_id(Path::new("firefox.desktop")).as_deref(),
        Some("firefox")
    );
    assert_eq!(desktop_id(Path::new("mimeinfo.cache")), None);
}

#[test]
fn o_primeiro_id_vence_mesmo_quando_esconde_o_do_sistema() {
    let dir = tempfile::tempdir().unwrap();
    let user = dir.path().join("user");
    let system = dir.path().join("system");
    fs::create_dir_all(user.join("kde")).unwrap();
    fs::create_dir_all(&system).unwrap();
    let app = |name: &str| format!("[Desktop Entry]\nType=Application\nName={name}\nExec={name}\n");
    fs::write(user.join("editor.desktop"), app("Meu editor")).unwrap();
    fs::write(system.join("editor.desktop"), app("Editor do sistema")).unwrap();
    fs::write(user.join("quebrado.desktop"), "sem grupo").unwrap();
    fs::write(system.join("quebrado.desktop"), app("Escondido")).unwrap();
    fs::write(user.join("kde/dolphin.desktop"), app("Dolphin")).unwrap();
    let entries = entries_in(&[user, system]);
    let names: Vec<_> = entries
        .iter()
        .map(|(id, entry)| (id.as_str(), entry.name.get("C").unwrap_or_default()))
        .collect();
    assert_eq!(
        names,
        [("editor", "Meu editor"), ("kde-dolphin", "Dolphin")]
    );
}
