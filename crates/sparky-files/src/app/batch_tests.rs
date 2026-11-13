use std::fs;

use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::state::Mode;
use super::test_scene::Cena;
use crate::view::batch_preview::preview;

fn com_tres() -> Cena {
    let cena = Cena::nova();
    let escrito = fs::write(cena.work().join("b.txt"), "b")
        .and_then(|()| fs::write(cena.work().join("c.md"), "c"));
    assert!(escrito.is_ok());
    let mut cena = cena;
    cena.mandar(Message::FolderChanged);
    for nome in ["a.txt", "b.txt", "c.md"] {
        cena.files.listing.select_name(nome);
        cena.files.listing.toggle_mark();
    }
    cena
}

fn nomes(cena: &Cena) -> Vec<String> {
    cena.files
        .listing
        .visible()
        .map(|entry| entry.name.clone())
        .collect()
}

fn aviso(cena: &Cena) -> String {
    cena.files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned())
        .unwrap_or_default()
}

#[test]
fn r_com_varios_marcados_renomeia_em_lote_na_ordem_da_lista() {
    let mut cena = com_tres();
    cena.tecla("r");
    assert!(matches!(&cena.files.mode, Mode::BatchRename(nomes) if nomes.len() == 3));
    cena.mandar(Message::Typed("Nota {n}".to_owned()));
    cena.mandar(Message::Submit);
    assert_eq!(
        nomes(&cena),
        ["Fotos", "Nota 1.txt", "Nota 2.txt", "Nota 3.md"]
    );
    assert_eq!(
        fs::read_to_string(cena.work().join("Nota 2.txt"))
            .ok()
            .as_deref(),
        Some("b")
    );
    assert_eq!(aviso(&cena), "3 itens renomeados");
    assert_eq!(cena.files.listing.marked_count(), 0);
}

#[test]
fn ctrl_z_devolve_todos_os_nomes_de_uma_vez() {
    let mut cena = com_tres();
    cena.tecla("r");
    cena.mandar(Message::Typed(". -> _".to_owned()));
    cena.mandar(Message::Submit);
    assert_eq!(nomes(&cena), ["Fotos", "a_txt", "b_txt", "c_md"]);
    cena.mandar(Message::Key(Key::Character("z".into()), Modifiers::CTRL));
    assert_eq!(nomes(&cena), ["Fotos", "a.txt", "b.txt", "c.md"]);
}

#[test]
fn nome_ja_ocupado_recusa_tudo_sem_mexer_em_nada() {
    let mut cena = com_tres();
    cena.tecla("r");
    cena.mandar(Message::Typed("c.md -> Fotos".to_owned()));
    let texto = preview(&cena.files, &["c.md".to_owned()]);
    assert_eq!(texto, "Já existe um item chamado Fotos");
    cena.mandar(Message::Submit);
    assert_eq!(nomes(&cena), ["Fotos", "a.txt", "b.txt", "c.md"]);
    assert_eq!(aviso(&cena), "Já existe um item chamado Fotos");
}

#[test]
fn a_previa_mostra_os_primeiros_nomes_e_quantos_faltam() {
    let mut cena = com_tres();
    cena.tecla("r");
    let nomes = vec!["a.txt".to_owned(), "b.txt".to_owned(), "c.md".to_owned()];
    assert_eq!(
        preview(&cena.files, &nomes),
        "{n} numera · antigo -> novo troca um trecho"
    );
    cena.mandar(Message::Typed("Viagem {n}".to_owned()));
    assert_eq!(
        preview(&cena.files, &nomes),
        "a.txt → Viagem 1.txt · b.txt → Viagem 2.txt · +1"
    );
}

#[test]
fn um_unico_item_continua_na_renomeacao_simples() {
    let mut cena = Cena::nova();
    cena.files.listing.select_name("a.txt");
    cena.tecla("r");
    assert_eq!(cena.files.mode, Mode::Rename("a.txt".to_owned()));
}

#[test]
fn renomear_em_cadeia_mantem_cada_favorito_na_sua_pasta() {
    let cena = Cena::nova();
    let criado =
        fs::create_dir(cena.work().join("a")).and_then(|()| fs::create_dir(cena.work().join("aa")));
    assert!(criado.is_ok());
    let mut cena = cena;
    cena.mandar(Message::FolderChanged);
    let sidebar = cena.files.services.sidebar.clone();
    assert!(sidebar.favorite(&cena.work().join("a"), true).is_ok());
    for nome in ["a", "aa"] {
        cena.files.listing.select_name(nome);
        cena.files.listing.toggle_mark();
    }
    cena.tecla("r");
    cena.mandar(Message::Typed("a -> aa".to_owned()));
    cena.mandar(Message::Submit);
    assert!(cena.work().join("aaaa").is_dir());
    assert!(sidebar.is_favorite(&cena.work().join("aa")));
    assert!(!sidebar.is_favorite(&cena.work().join("aaaa")));
    cena.concluir(Message::Key(Key::Character("z".into()), Modifiers::CTRL));
    assert!(cena.work().join("a").is_dir());
    assert!(sidebar.is_favorite(&cena.work().join("a")));
}
