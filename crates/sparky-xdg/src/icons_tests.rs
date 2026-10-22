use std::fs;
use std::path::Path;

use super::icons::{IconTheme, theme_from_settings};

fn theme(root: &Path, name: &str, inherits: &str, directory: &str) {
    fs::create_dir_all(root.join(name).join(directory)).unwrap();
    fs::write(
        root.join(name).join("index.theme"),
        format!("[Icon Theme]\nDirectories={directory}\nInherits={inherits}\n"),
    )
    .unwrap();
}

#[test]
fn respeita_tema_do_usuario_diretorios_do_indice_e_heranca() {
    let dir = tempfile::tempdir().unwrap();
    theme(dir.path(), "Yaru-sage", "Yaru", "48x48/places");
    theme(dir.path(), "Yaru", "hicolor", "scalable/mimetypes");
    let folder = dir.path().join("Yaru-sage/48x48/places/folder.png");
    let pdf = dir.path().join("Yaru/scalable/mimetypes/application-pdf.svg");
    fs::write(&folder, "png").unwrap();
    fs::write(&pdf, "svg").unwrap();
    let icons = IconTheme::new(vec![dir.path().to_path_buf()], "Yaru-sage");
    assert_eq!(icons.resolve(&["folder"]), Some(folder));
    assert_eq!(icons.resolve(&["application-pdf", "text-x-generic"]), Some(pdf));
    assert_eq!(icons.resolve(&["inexistente"]), None);
}

#[test]
fn heranca_circular_termina_e_raiz_do_usuario_tem_prioridade() {
    let dir = tempfile::tempdir().unwrap();
    let user = dir.path().join("user");
    let system = dir.path().join("system");
    theme(&user, "custom", "custom,parent", "64x64/places");
    theme(&system, "custom", "parent", "64x64/places");
    theme(&system, "parent", "custom", "64x64/places");
    let chosen = user.join("custom/64x64/places/folder.svg");
    fs::write(&chosen, "svg").unwrap();
    fs::write(system.join("custom/64x64/places/folder.svg"), "svg").unwrap();
    let icons = IconTheme::new(vec![user, system], "custom");
    assert_eq!(icons.resolve(&["folder"]), Some(chosen));
    assert_eq!(icons.resolve(&["../../arquivo"]), None);
}

#[test]
fn hicolor_e_usado_quando_o_tema_nao_existe() {
    let dir = tempfile::tempdir().unwrap();
    theme(dir.path(), "hicolor", "", "48x48/mimetypes");
    let fallback = dir.path().join("hicolor/48x48/mimetypes/text-x-generic.svg");
    fs::write(&fallback, "svg").unwrap();
    let icons = IconTheme::new(vec![dir.path().to_path_buf()], "ausente");
    assert_eq!(icons.resolve(&["sem-tipo", "text-x-generic"]), Some(fallback));
}

#[test]
fn configuracao_do_gtk_ignora_outras_secoes_e_valores_vazios() {
    assert_eq!(
        theme_from_settings("[Other]\ngtk-icon-theme-name=Outro\n[Settings]\ngtk-icon-theme-name=\"Yaru-sage\""),
        Some("Yaru-sage".to_owned())
    );
    assert_eq!(theme_from_settings("[Settings]\ngtk-icon-theme-name=\n"), None);
    assert_eq!(
        theme_from_settings("[Settings]\ngtk-icon-theme-name = Yaru-sage\n"),
        Some("Yaru-sage".to_owned())
    );
}
