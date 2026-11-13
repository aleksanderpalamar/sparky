use std::fs;

use iced::keyboard::{Key, Modifiers};
use sparky_ui::notice::Notice;

use super::test_scene::Cena;
use super::test_sources::AreaFalsa;
use crate::app::message::Message;
use crate::domain::image::ImageMime;

#[test]
fn ctrl_c_em_png_publica_imagem_e_preserva_copia_interna() {
    let mut cena = Cena::nova();
    let image = cena.work().join("foto.png");
    fs::write(&image, b"png").unwrap();
    cena.files.refresh();
    cena.files.listing.select_name("foto.png");

    cena.mandar(Message::Key(Key::Character("c".into()), Modifiers::CTRL));

    assert_eq!(cena.area.imagens(), vec![(image.clone(), ImageMime::Png)]);
    assert_eq!(
        cena.files.clip.as_ref().map(|clip| clip.paths().len()),
        Some(1)
    );
    cena.concluir(Message::Key(Key::Character("v".into()), Modifiers::CTRL));
    assert!(cena.work().join("foto (cópia).png").exists());
}

#[test]
fn arquivo_comum_e_selecao_multipla_nao_publicam_imagem() {
    let mut cena = Cena::nova();
    fs::write(cena.work().join("foto.png"), b"png").unwrap();
    cena.files.refresh();
    cena.files.listing.select_name("a.txt");
    cena.mandar(Message::Key(Key::Character("c".into()), Modifiers::CTRL));
    assert!(cena.area.imagens().is_empty());

    cena.files.listing.select_name("foto.png");
    cena.files.listing.toggle_mark();
    cena.files.listing.select_name("a.txt");
    cena.files.listing.toggle_mark();
    cena.mandar(Message::Key(Key::Character("c".into()), Modifiers::CTRL));
    assert!(cena.area.imagens().is_empty());
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
        assert_eq!(cena.area.imagens().last(), Some(&(path, mime)));
    }
}

#[test]
fn falha_do_clipboard_externo_preserva_copia_interna_e_informa_erro() {
    let mut cena = Cena::nova();
    let path = cena.work().join("foto.png");
    fs::write(&path, b"png").unwrap();
    cena.files.services.clipboard = Box::new(AreaFalsa {
        falha: true,
        ..AreaFalsa::default()
    });
    cena.files.refresh();
    cena.files.listing.select_name("foto.png");

    cena.mandar(Message::Key(Key::Character("c".into()), Modifiers::CTRL));

    assert_eq!(
        cena.files.clip.as_ref().map(|clip| clip.paths().len()),
        Some(1)
    );
    assert!(matches!(cena.files.notice, Some(Notice::Error(_))));
}
