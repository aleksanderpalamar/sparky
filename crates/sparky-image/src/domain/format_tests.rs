use std::path::Path;

use super::format::Format;

#[test]
fn reconhece_formatos_pela_extensao_sem_diferenciar_maiusculas() {
    assert_eq!(Format::from_path(Path::new("foto.JPG")), Some(Format::Jpeg));
    assert_eq!(Format::from_path(Path::new("a/b.jpeg")), Some(Format::Jpeg));
    assert_eq!(Format::from_path(Path::new("tela.png")), Some(Format::Png));
    assert_eq!(Format::from_path(Path::new("x.webp")), Some(Format::Webp));
    assert_eq!(Format::from_path(Path::new("x.gif")), Some(Format::Gif));
    assert_eq!(Format::from_path(Path::new("x.bmp")), Some(Format::Bmp));
    assert_eq!(Format::from_path(Path::new("x.TIF")), Some(Format::Tiff));
    assert_eq!(Format::from_path(Path::new("x.tiff")), Some(Format::Tiff));
}

#[test]
fn outros_arquivos_nao_sao_imagens() {
    assert_eq!(Format::from_path(Path::new("notas.txt")), None);
    assert_eq!(Format::from_path(Path::new("sem-extensao")), None);
    assert_eq!(Format::from_path(Path::new(".png")), None);
}

#[test]
fn arquivo_desktop_e_padroes_do_sparky_cobrem_todos_os_formatos() {
    let desktop = include_str!("../../../../data/applications/sparky-image.desktop");
    let defaults = include_str!("../../../../data/applications/sparky-mimeapps.list");
    for format in Format::ALL {
        let mime = format.mime();
        assert!(desktop.contains(&format!("{mime};")), "{mime} no .desktop");
        assert!(
            defaults.contains(&format!("{mime}=sparky-image.desktop")),
            "{mime} nos padrões"
        );
    }
}
