use std::fs;
use std::path::{Path, PathBuf};

use super::icons::IconTheme;

fn index(root: &Path, theme: &str, contents: &str) {
    fs::create_dir_all(root.join(theme)).unwrap();
    fs::write(root.join(theme).join("index.theme"), contents).unwrap();
}

fn icon(root: &Path, theme: &str, directory: &str, file: &str) -> PathBuf {
    let folder = root.join(theme).join(directory);
    fs::create_dir_all(&folder).unwrap();
    let path = folder.join(file);
    fs::write(&path, "x").unwrap();
    path
}

#[test]
fn pasta_do_usuario_sem_indice_usa_as_pastas_do_indice_do_sistema() {
    let dir = tempfile::tempdir().unwrap();
    let user = dir.path().join("user");
    let system = dir.path().join("system");
    index(
        &system,
        "hicolor",
        "[Icon Theme]\nDirectories=512x512/apps\n",
    );
    let studio = icon(&user, "hicolor", "512x512/apps", "lm-studio.png");
    let icons = IconTheme::with_size(vec![user, system], "hicolor", 32);
    assert_eq!(icons.resolve(&["lm-studio"]), Some(studio));
}

#[test]
fn tamanho_exato_em_escala_um_vence_o_desenho_menor_ampliado() {
    let dir = tempfile::tempdir().unwrap();
    index(
        dir.path(),
        "Yaru",
        "[Icon Theme]\nDirectories=16x16@2x/apps,32x32/apps\n\
         [16x16@2x/apps]\nSize=16\nScale=2\n[32x32/apps]\nSize=32\n",
    );
    icon(dir.path(), "Yaru", "16x16@2x/apps", "org.gnome.Evince.png");
    let sharp = icon(dir.path(), "Yaru", "32x32/apps", "org.gnome.Evince.png");
    let icons = IconTheme::with_size(vec![dir.path().to_path_buf()], "Yaru", 32);
    assert_eq!(icons.resolve(&["org.gnome.Evince"]), Some(sharp));
}

#[test]
fn vetor_vence_bitmap_muito_maior_que_o_pedido() {
    let dir = tempfile::tempdir().unwrap();
    index(
        dir.path(),
        "hicolor",
        "[Icon Theme]\nDirectories=512x512/apps,scalable/apps\n\
         [512x512/apps]\nSize=512\n[scalable/apps]\nSize=128\nType=Scalable\n",
    );
    icon(dir.path(), "hicolor", "512x512/apps", "nordvpn.png");
    let vector = icon(dir.path(), "hicolor", "scalable/apps", "nordvpn.svg");
    let icons = IconTheme::with_size(vec![dir.path().to_path_buf()], "hicolor", 32);
    assert_eq!(icons.resolve(&["nordvpn"]), Some(vector));
}

#[test]
fn tamanho_vem_da_secao_quando_o_nome_da_pasta_nao_o_diz() {
    let dir = tempfile::tempdir().unwrap();
    index(
        dir.path(),
        "breeze",
        "[Icon Theme]\nDirectories=apps/16,apps/48\n\
         [apps/16]\nSize=16\n[apps/48]\nSize=48\n",
    );
    icon(dir.path(), "breeze", "apps/16", "kate.svg");
    let large = icon(dir.path(), "breeze", "apps/48", "kate.svg");
    let icons = IconTheme::with_size(vec![dir.path().to_path_buf()], "breeze", 48);
    assert_eq!(icons.resolve(&["kate"]), Some(large));
}

#[test]
fn nome_com_extensao_acha_o_icone_pelo_nome_ou_pelo_arquivo_literal() {
    let dir = tempfile::tempdir().unwrap();
    index(
        dir.path(),
        "hicolor",
        "[Icon Theme]\nDirectories=scalable/apps\n[scalable/apps]\nType=Scalable\n",
    );
    let kvantum = icon(dir.path(), "hicolor", "scalable/apps", "kvantum.svg");
    let literal = icon(dir.path(), "hicolor", "scalable/apps", "nordvpn-gui.png");
    let icons = IconTheme::with_size(vec![dir.path().to_path_buf()], "hicolor", 32);
    assert_eq!(icons.resolve(&["kvantum.png"]), Some(kvantum));
    assert_eq!(icons.resolve(&["nordvpn-gui.png"]), Some(literal));
}
