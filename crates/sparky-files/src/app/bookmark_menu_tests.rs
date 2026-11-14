use std::fs;
use std::path::PathBuf;

use super::message::Message;
use super::test_scene::Cena;
use crate::adapters::bookmark_file::BookmarkFile;
use crate::domain::bookmarks::Bookmark;
use crate::domain::context::{Action, Target};
use crate::domain::tag_color::TagColor;
use crate::ports::BookmarkStore;

fn arquivo(cena: &Cena) -> PathBuf {
    cena.dir.path().join("config/sparky/bookmarks")
}

fn com_marcadores(nomes: &[&str]) -> Cena {
    let mut cena = Cena::nova();
    let lista: Vec<_> = nomes
        .iter()
        .map(|nome| {
            let pasta = cena.work().join(nome);
            assert!(fs::create_dir_all(&pasta).is_ok());
            assert!(fs::write(pasta.join("dentro.txt"), "x").is_ok());
            Bookmark::local(pasta)
        })
        .collect();
    assert!(BookmarkFile::new(arquivo(&cena)).save(&lista).is_ok());
    cena.mandar(Message::Tick);
    cena
}

fn gravados(cena: &Cena) -> Vec<(String, Option<TagColor>)> {
    BookmarkFile::new(arquivo(cena))
        .load()
        .unwrap_or_default()
        .iter()
        .map(|marcador| (marcador.name(), marcador.color))
        .collect()
}

fn escolher(cena: &mut Cena, indice: usize, acao: Action) {
    cena.mandar(Message::ContextBookmark(indice));
    cena.mandar(Message::ContextAction(acao));
}

fn pasta_intacta(cena: &Cena, nome: &str) -> bool {
    fs::read_to_string(cena.work().join(nome).join("dentro.txt")).is_ok_and(|texto| texto == "x")
}

#[test]
fn botao_direito_num_marcador_abre_o_menu_dele() {
    let mut cena = com_marcadores(&["A", "B"]);
    cena.mandar(Message::ContextBookmark(1));
    let alvo = cena.files.context.as_ref().map(|menu| menu.target);
    assert_eq!(alvo, Some(Target::Bookmark(1)));
}

#[test]
fn escolher_uma_cor_pinta_o_marcador_na_hora_e_grava() {
    let mut cena = com_marcadores(&["A", "B"]);
    escolher(&mut cena, 1, Action::Tag(Some(TagColor::Green)));
    let cor = cena
        .files
        .bookmarks
        .items
        .get(1)
        .and_then(|marcador| marcador.color);
    assert_eq!(cor, Some(TagColor::Green));
    assert!(cena.files.context.is_none());
    assert_eq!(
        gravados(&cena),
        [
            ("A".to_owned(), None),
            ("B".to_owned(), Some(TagColor::Green))
        ]
    );
    assert!(pasta_intacta(&cena, "B"));
}

#[test]
fn sem_cor_volta_ao_padrao_e_a_cor_sobrevive_a_releitura() {
    let mut cena = com_marcadores(&["A"]);
    escolher(&mut cena, 0, Action::Tag(Some(TagColor::Pink)));
    cena.mandar(Message::Tick);
    let cor = cena
        .files
        .bookmarks
        .items
        .get(0)
        .and_then(|marcador| marcador.color);
    assert_eq!(cor, Some(TagColor::Pink));
    escolher(&mut cena, 0, Action::Tag(None));
    assert_eq!(gravados(&cena), [("A".to_owned(), None)]);
}

#[test]
fn remover_tira_da_lateral_e_do_arquivo_sem_tocar_na_pasta() {
    let mut cena = com_marcadores(&["A", "B", "C"]);
    escolher(&mut cena, 1, Action::Tag(Some(TagColor::Red)));
    escolher(&mut cena, 0, Action::RemoveBookmark);
    cena.mandar(Message::Tick);
    let nomes: Vec<_> = cena
        .files
        .bookmarks
        .items
        .local()
        .map(|(_, marcador)| marcador.name())
        .collect();
    assert_eq!(nomes, ["B", "C"]);
    assert_eq!(
        gravados(&cena),
        [
            ("B".to_owned(), Some(TagColor::Red)),
            ("C".to_owned(), None)
        ]
    );
    assert!(pasta_intacta(&cena, "A"));
    let aviso = cena
        .files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned());
    assert_eq!(aviso.as_deref(), Some("“A” removido dos marcadores"));
}

#[test]
fn a_acao_vale_para_o_marcador_clicado_mesmo_se_o_arquivo_mudou_por_fora() {
    let mut cena = com_marcadores(&["A", "B"]);
    cena.mandar(Message::ContextBookmark(1));
    let mut externa = BookmarkFile::new(arquivo(&cena)).load().unwrap_or_default();
    externa.remove(0);
    assert!(BookmarkFile::new(arquivo(&cena)).save(&externa).is_ok());
    cena.mandar(Message::Tick);
    cena.mandar(Message::ContextAction(Action::RemoveBookmark));
    assert!(gravados(&cena).is_empty());
    assert!(cena.files.bookmarks.items.items().is_empty());
}
