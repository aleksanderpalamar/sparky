use std::fs;
use std::io;
use std::path::Path;

use iced::keyboard::{Key, Modifiers};
use sparky_ui::notice::Notice;

use super::test_scene::Cena;
use crate::app::message::Message;
use crate::domain::image::ImageMime;
use crate::ports::ImageClipboard;

struct ClipboardIndisponivel;

impl ImageClipboard for ClipboardIndisponivel {
    fn publish(&self, _path: &Path, _mime: ImageMime) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::NotFound, "wl-copy ausente"))
    }
}

#[test]
fn ctrl_c_em_png_publica_imagem_e_preserva_copia_interna() {
    let mut cena = Cena::nova();
    let image = cena.work().join("foto.png");
    fs::write(&image, b"png").unwrap();
    cena.files.refresh();
    cena.files.listing.select_name("foto.png");

    cena.mandar(Message::Key(Key::Character("c".into()), Modifiers::CTRL));

    assert_eq!(
        *cena.imagens.0.borrow(),
        vec![(image.clone(), ImageMime::Png)]
    );
    assert_eq!(
        cena.files.clip.as_ref().map(|clip| clip.paths().len()),
        Some(1)
    );
    cena.mandar(Message::Key(Key::Character("v".into()), Modifiers::CTRL));
    assert!(cena.work().join("foto (cópia).png").exists());
}

#[test]
fn arquivo_comum_e_selecao_multipla_nao_publicam_imagem() {
    let mut cena = Cena::nova();
    fs::write(cena.work().join("foto.png"), b"png").unwrap();
    cena.files.refresh();
    cena.files.listing.select_name("a.txt");
    cena.mandar(Message::Key(Key::Character("c".into()), Modifiers::CTRL));
    assert!(cena.imagens.0.borrow().is_empty());

    cena.files.listing.select_name("foto.png");
    cena.files.listing.toggle_mark();
    cena.files.listing.select_name("a.txt");
    cena.files.listing.toggle_mark();
    cena.mandar(Message::Key(Key::Character("c".into()), Modifiers::CTRL));
    assert!(cena.imagens.0.borrow().is_empty());
}

#[test]
fn jpeg_e_webp_publicam_os_formatos_correspondentes() {
    let mut cena = Cena::nova();
    for (name, mime) in [
        ("foto.jpg", ImageMime::Jpeg),
        ("foto.webp", ImageMime::Webp),
    ] {
        let path = cena.work().join(name);
        fs::write(&path, b"imagem").unwrap();
        cena.files.refresh();
        cena.files.listing.select_name(name);
        cena.mandar(Message::Key(Key::Character("c".into()), Modifiers::CTRL));
        assert_eq!(cena.imagens.0.borrow().last(), Some(&(path, mime)));
    }
}

#[test]
fn falha_do_clipboard_externo_preserva_copia_interna_e_informa_erro() {
    let mut cena = Cena::nova();
    let path = cena.work().join("foto.png");
    fs::write(&path, b"png").unwrap();
    cena.files.services.image_clipboard = Box::new(ClipboardIndisponivel);
    cena.files.refresh();
    cena.files.listing.select_name("foto.png");

    cena.mandar(Message::Key(Key::Character("c".into()), Modifiers::CTRL));

    assert_eq!(
        cena.files.clip.as_ref().map(|clip| clip.paths().len()),
        Some(1)
    );
    assert!(matches!(cena.files.notice, Some(Notice::Error(_))));
}
