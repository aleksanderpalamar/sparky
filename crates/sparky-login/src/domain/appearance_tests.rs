use std::path::{Path, PathBuf};

use crate::domain::appearance::{Sources, sources, wallpaper};

const CASA: &str = "/home/palamar";

#[test]
fn le_o_sparky_toml_e_as_cores_da_pasta_de_configuracao() {
    assert_eq!(
        sources(Path::new(CASA), None),
        Sources {
            config: PathBuf::from("/home/palamar/.config/sparky/sparky.toml"),
            colors: PathBuf::from("/home/palamar/.config/sparky/colors.toml")
        }
    );
    assert_eq!(
        sources(Path::new(CASA), Some("/cfg")).colors,
        PathBuf::from("/cfg/sparky/colors.toml")
    );
}

#[test]
fn a_imagem_e_a_do_login_e_sem_ela_a_da_area_de_trabalho() {
    let ambos = "[general]\nwallpaper = \"/w/a.jpg\"\n\n[login]\nwallpaper = \"/w/b.jpg\"\n";
    assert_eq!(
        wallpaper(Some(ambos), Path::new(CASA)),
        Some(PathBuf::from("/w/b.jpg"))
    );
    assert_eq!(
        wallpaper(
            Some("[general]\nwallpaper = \"/w/a.jpg\"\n"),
            Path::new(CASA)
        ),
        Some(PathBuf::from("/w/a.jpg"))
    );
}

#[test]
fn til_vai_para_a_pasta_pessoal() {
    let texto = "[login]\nwallpaper = \"~/Imagens/fundo.png\"\n";
    assert_eq!(
        wallpaper(Some(texto), Path::new(CASA)),
        Some(PathBuf::from("/home/palamar/Imagens/fundo.png"))
    );
}

#[test]
fn sem_escolha_texto_vazio_ou_arquivo_invalido_nao_ha_imagem() {
    for texto in [
        None,
        Some(""),
        Some("[login]\nwallpaper = \" \"\n"),
        Some("{quebrado"),
    ] {
        assert_eq!(wallpaper(texto, Path::new(CASA)), None, "{texto:?}");
    }
}
