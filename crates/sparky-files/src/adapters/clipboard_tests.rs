use std::path::PathBuf;

use super::clipboard::entries;
use super::clipboard_request::{Content, parse};
use crate::domain::clip::{Clip, ClipOffer};
use crate::domain::image::ImageMime;

#[test]
fn copia_de_imagem_oferece_arquivos_texto_e_a_propria_imagem() {
    let foto = PathBuf::from("/home/ana/foto.png");
    let offer = ClipOffer {
        clip: Clip::Copy(vec![foto.clone()]),
        image: Some((foto.clone(), ImageMime::Png)),
    };
    let servido = parse(&entries(&offer).to_string()).unwrap();
    let tipos: Vec<&str> = servido.iter().map(|(mime, _)| mime.as_str()).collect();
    assert_eq!(
        tipos,
        [
            "x-special/gnome-copied-files",
            "text/uri-list",
            "text/plain;charset=utf-8",
            "UTF8_STRING",
            "image/png"
        ]
    );
    assert_eq!(
        servido[0].1,
        Content::Text("copy\nfile:///home/ana/foto.png".to_owned())
    );
    assert_eq!(servido[4].1, Content::File(foto));
}

#[test]
fn recorte_marca_o_formato_do_kde() {
    let offer = ClipOffer {
        clip: Clip::Cut(vec![PathBuf::from("/a")]),
        image: None,
    };
    let servido = parse(&entries(&offer).to_string()).unwrap();
    assert!(servido.contains(&(
        "application/x-kde-cutselection".to_owned(),
        Content::Text("1".to_owned())
    )));
}

#[test]
fn entrada_invalida_do_servidor_e_recusada() {
    assert!(parse("[{\"mime\": \"text/plain\"}]").is_none());
    assert!(parse("{}").is_none());
    assert_eq!(parse("[]"), Some(Vec::new()));
}
