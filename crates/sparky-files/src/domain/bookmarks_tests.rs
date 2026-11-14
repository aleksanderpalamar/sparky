use std::path::{Path, PathBuf};

use super::bookmarks::{Added, Bookmark, Bookmarks, Target};
use super::tag_color::TagColor;

fn marcadores(caminhos: &[&str]) -> Bookmarks {
    Bookmarks::new(
        caminhos
            .iter()
            .map(|caminho| Bookmark::local(PathBuf::from(caminho)))
            .collect(),
    )
}

fn caminhos(lista: &Bookmarks) -> Vec<String> {
    lista
        .items()
        .iter()
        .map(|marcador| marcador.name())
        .collect()
}

#[test]
fn adicionar_a_mesma_pasta_duas_vezes_nao_duplica() {
    let mut lista = marcadores(&["/h/Projetos"]);
    assert_eq!(lista.add(PathBuf::from("/h/Projetos/")), Added::Duplicate);
    assert_eq!(lista.add(PathBuf::from("/h/Fotos")), Added::New);
    assert_eq!(caminhos(&lista), ["Projetos", "Fotos"]);
}

#[test]
fn reordenar_move_o_marcador_para_a_posicao_do_alvo() {
    let mut lista = marcadores(&["/a", "/b", "/c"]);
    assert!(lista.reorder(0, 2));
    assert_eq!(caminhos(&lista), ["b", "c", "a"]);
    assert!(lista.reorder(2, 0));
    assert_eq!(caminhos(&lista), ["a", "b", "c"]);
}

#[test]
fn reordenar_para_o_mesmo_lugar_ou_fora_da_lista_nao_muda_nada() {
    let mut lista = marcadores(&["/a", "/b"]);
    assert!(!lista.reorder(1, 1));
    assert!(!lista.reorder(0, 5));
    assert_eq!(caminhos(&lista), ["a", "b"]);
}

#[test]
fn o_nome_usa_o_rotulo_e_so_os_locais_aparecem() {
    let lista = Bookmarks::new(vec![
        Bookmark {
            target: Target::Remote("sftp://servidor/".to_owned()),
            label: None,
            color: None,
        },
        Bookmark {
            target: Target::Local(PathBuf::from("/h/proj")),
            label: Some("Trabalho".to_owned()),
            color: Some(TagColor::Blue),
        },
    ]);
    let locais: Vec<_> = lista
        .local()
        .map(|(indice, marcador)| (indice, marcador.name()))
        .collect();
    assert_eq!(locais, [(1, "Trabalho".to_owned())]);
    assert!(lista.contains(Path::new("/h/proj")));
    assert_eq!(lista.find(Path::new("/h/proj")), Some(1));
    assert_eq!(lista.find(Path::new("/h/outra")), None);
}

#[test]
fn pintar_troca_so_a_cor_e_sem_cor_volta_ao_padrao() {
    let mut lista = marcadores(&["/a", "/b"]);
    assert!(lista.paint(1, Some(TagColor::Red)));
    assert!(!lista.paint(1, Some(TagColor::Red)));
    assert_eq!(
        lista.get(1).and_then(|marcador| marcador.color),
        Some(TagColor::Red)
    );
    assert_eq!(lista.get(0).and_then(|marcador| marcador.color), None);
    assert!(lista.paint(1, None));
    assert_eq!(lista.get(1).and_then(|marcador| marcador.color), None);
    assert!(!lista.paint(5, Some(TagColor::Blue)));
}

#[test]
fn remover_tira_so_o_marcador_escolhido() {
    let mut lista = marcadores(&["/a", "/b", "/c"]);
    let removido = lista.remove(1).map(|marcador| marcador.name());
    assert_eq!(removido.as_deref(), Some("b"));
    assert_eq!(caminhos(&lista), ["a", "c"]);
    assert!(lista.remove(9).is_none());
}
