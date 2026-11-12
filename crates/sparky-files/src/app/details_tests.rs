use std::fs;

use iced::Size;

use super::message::Message;
use super::test_scene::Cena;
use crate::domain::entry::Meta;

fn conhecidos(cena: &Cena) -> usize {
    cena.files
        .listing
        .visible()
        .filter(|entry| entry.meta != Meta::Pending)
        .count()
}

fn pasta_grande() -> Cena {
    let mut cena = Cena::nova();
    let grande = cena.work().join("Grande");
    fs::create_dir(&grande).unwrap();
    for indice in 0..2000 {
        fs::write(grande.join(format!("arquivo-{indice:04}.txt")), "abc").unwrap();
    }
    cena.files.resize(Size::new(900.0, 560.0));
    assert!(cena.files.go(grande));
    cena
}

#[test]
fn pasta_grande_consulta_so_as_linhas_visiveis() {
    let cena = pasta_grande();
    let visiveis = cena.files.visible_range().len();
    assert_eq!(cena.files.listing.len(), 2000);
    assert_eq!(conhecidos(&cena), visiveis);
    assert_eq!(
        cena.files.listing.get(0).and_then(|entry| entry.size()),
        Some(3)
    );
}

#[test]
fn rolar_preenche_as_linhas_que_aparecem() {
    let mut cena = pasta_grande();
    let antes = conhecidos(&cena);
    cena.mandar(Message::Scrolled(10));
    let range = cena.files.visible_range();
    assert!(range.start > 0);
    assert!(range.clone().all(|index| {
        cena.files
            .listing
            .get(index)
            .is_some_and(|entry| entry.size().is_some())
    }));
    assert!(conhecidos(&cena) > antes);
    assert!(conhecidos(&cena) < 2000);
}
