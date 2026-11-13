use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::batch_rename::rename_all;

fn ambiente(arquivos: &[(&str, &str)]) -> (TempDir, LocalDisk) {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("sem pasta temporária")
    };
    for (nome, texto) in arquivos {
        assert!(fs::write(dir.path().join(nome), texto).is_ok());
    }
    let disk = LocalDisk::new(HomeTrash::new(dir.path().join("Trash")));
    (dir, disk)
}

fn par(dir: &Path, de: &str, para: &str) -> (PathBuf, PathBuf) {
    (dir.join(de), dir.join(para))
}

fn ler(dir: &Path, nome: &str) -> String {
    fs::read_to_string(dir.join(nome)).unwrap_or_default()
}

fn itens(dir: &Path) -> Vec<String> {
    let mut nomes: Vec<String> = fs::read_dir(dir)
        .map(|lista| {
            lista
                .filter_map(Result::ok)
                .map(|item| item.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    nomes.sort();
    nomes
}

#[test]
fn troca_de_nomes_entre_dois_itens_funciona() {
    let (dir, disk) = ambiente(&[("a", "A"), ("b", "B")]);
    let pares = [par(dir.path(), "a", "b"), par(dir.path(), "b", "a")];
    assert!(rename_all(&pares, &disk).is_ok());
    assert_eq!(
        (ler(dir.path(), "a"), ler(dir.path(), "b")),
        ("B".into(), "A".into())
    );
    assert_eq!(itens(dir.path()), ["a", "b"]);
}

#[test]
fn cadeia_de_nomes_nao_sobrescreve_ninguem() {
    let (dir, disk) = ambiente(&[("1", "um"), ("2", "dois")]);
    let pares = [par(dir.path(), "1", "2"), par(dir.path(), "2", "3")];
    assert!(rename_all(&pares, &disk).is_ok());
    assert_eq!(
        (ler(dir.path(), "2"), ler(dir.path(), "3")),
        ("um".into(), "dois".into())
    );
    assert_eq!(itens(dir.path()), ["2", "3"]);
}

#[test]
fn falha_no_meio_desfaz_o_que_ja_tinha_mudado() {
    let (dir, disk) = ambiente(&[("a", "A")]);
    let pares = [par(dir.path(), "a", "x"), par(dir.path(), "sumiu", "y")];
    let erro = rename_all(&pares, &disk)
        .err()
        .map(|falha| falha.to_string());
    assert!(erro.is_some_and(|texto| texto.starts_with("sumiu:")));
    assert_eq!(itens(dir.path()), ["a"]);
}

#[test]
fn itens_que_nao_mudam_de_nome_sao_ignorados() {
    let (dir, disk) = ambiente(&[("a", "A"), ("b", "B")]);
    let pares = [par(dir.path(), "a", "a"), par(dir.path(), "b", "c")];
    assert!(rename_all(&pares, &disk).is_ok());
    assert_eq!(itens(dir.path()), ["a", "c"]);
}
