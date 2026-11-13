use std::fs;

use super::test_scene::Cena;
use crate::domain::listing::Hidden;

#[test]
fn revelar_abre_a_pasta_e_marca_os_itens_pedidos() {
    let mut cena = Cena::nova();
    let fotos = cena.work().join("Fotos");
    let criado =
        fs::write(fotos.join("x.jpg"), "x").and_then(|()| fs::write(fotos.join("y.jpg"), "y"));
    assert!(criado.is_ok());
    cena.files
        .reveal(&[fotos.join("y.jpg"), fotos.join("x.jpg")]);
    assert_eq!(cena.files.location, fotos);
    assert_eq!(cena.selecionado().as_deref(), Some("y.jpg"));
    assert_eq!(cena.files.listing.chosen(), ["x.jpg", "y.jpg"]);
}

#[test]
fn um_unico_item_fica_so_selecionado_e_oculto_aparece() {
    let mut cena = Cena::nova();
    assert!(fs::write(cena.work().join(".env"), "x").is_ok());
    cena.files.reveal(&[cena.work().join(".env")]);
    assert_eq!(cena.files.listing.hidden(), Hidden::Shown);
    assert_eq!(cena.selecionado().as_deref(), Some(".env"));
    assert_eq!(cena.files.listing.marked_count(), 0);
}

#[test]
fn propriedades_mostram_o_item_revelado() {
    let mut cena = Cena::nova();
    let item = cena.work().join("a.txt");
    cena.files.reveal_properties(&item);
    assert_eq!(
        cena.files
            .details
            .as_ref()
            .map(|details| details.path.clone()),
        Some(item)
    );
}
