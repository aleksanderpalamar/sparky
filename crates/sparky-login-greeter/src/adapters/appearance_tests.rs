use std::fs;
use std::path::PathBuf;

use sparky_theme::{Palette, Rgb};

use crate::adapters::appearance::{palette_from, wallpaper};

#[test]
fn as_cores_copiadas_do_usuario_vem_primeiro() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    assert!(fs::write(dir.path().join("colors.toml"), "accent = \"#010203\"\n").is_ok());
    let padrao = vec![PathBuf::from("/nao/existe/colors.toml")];
    assert_eq!(palette_from(dir.path(), padrao).accent, Rgb::new(1, 2, 3));
}

#[test]
fn sem_copia_vale_o_tema_padrao() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    assert_eq!(palette_from(dir.path(), Vec::new()), Palette::TOKYO_NIGHT);
}

#[test]
fn a_imagem_copiada_so_vale_se_existir() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    assert_eq!(wallpaper(dir.path()), None);
    assert!(fs::write(dir.path().join("wallpaper"), b"imagem").is_ok());
    assert_eq!(wallpaper(dir.path()), Some(dir.path().join("wallpaper")));
}
