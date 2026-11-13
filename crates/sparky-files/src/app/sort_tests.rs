use std::fs;

use super::message::Message;
use super::test_scene::Cena;
use crate::domain::order::{Order, SortDirection, SortKey};

fn nomes(cena: &Cena) -> Vec<String> {
    cena.files
        .listing
        .visible()
        .map(|entry| entry.name.clone())
        .collect()
}

fn preparada() -> Cena {
    let cena = Cena::nova();
    let escrito = fs::write(cena.work().join("b.txt"), "um texto maior")
        .and_then(|()| fs::write(cena.work().join("c.md"), "x"));
    assert!(escrito.is_ok());
    let mut cena = cena;
    cena.mandar(Message::FolderChanged);
    cena
}

#[test]
fn s_troca_o_criterio_e_s_maiusculo_inverte() {
    let mut cena = preparada();
    assert_eq!(nomes(&cena), ["Fotos", "a.txt", "b.txt", "c.md"]);
    cena.tecla("s");
    assert_eq!(cena.files.listing.order().key, SortKey::Size);
    assert_eq!(nomes(&cena), ["Fotos", "c.md", "a.txt", "b.txt"]);
    cena.tecla("S");
    assert_eq!(nomes(&cena), ["Fotos", "b.txt", "a.txt", "c.md"]);
    let aviso = cena.files.notice.as_ref().map(|n| n.text().to_owned());
    assert_eq!(aviso.as_deref(), Some("Ordenado por tamanho ↓"));
}

#[test]
fn a_ordem_vale_ao_navegar_e_ao_reler_a_pasta() {
    let mut cena = preparada();
    cena.mandar(Message::Sort(SortKey::Size));
    cena.mandar(Message::Sort(SortKey::Size));
    let pasta = cena.work().join("Fotos");
    assert!(fs::write(pasta.join("p.txt"), "1").is_ok());
    assert!(fs::write(pasta.join("g.txt"), "1234567").is_ok());
    cena.files.go(pasta);
    assert_eq!(nomes(&cena), ["g.txt", "p.txt"]);
    assert!(fs::write(cena.files.location.join("m.txt"), "1234").is_ok());
    cena.mandar(Message::FolderChanged);
    assert_eq!(nomes(&cena), ["g.txt", "m.txt", "p.txt"]);
}

#[test]
fn clicar_no_titulo_escolhe_o_criterio_e_de_novo_inverte() {
    let mut cena = preparada();
    cena.mandar(Message::Sort(SortKey::Type));
    assert_eq!(
        cena.files.listing.order(),
        Order::new(SortKey::Type, SortDirection::Ascending)
    );
    assert_eq!(nomes(&cena), ["Fotos", "c.md", "a.txt", "b.txt"]);
    cena.mandar(Message::Sort(SortKey::Name));
    cena.mandar(Message::Sort(SortKey::Name));
    assert_eq!(nomes(&cena), ["Fotos", "c.md", "b.txt", "a.txt"]);
}
