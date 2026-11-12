use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

use crate::adapters::extract::extract;
use crate::domain::archive::Extraction;
use crate::domain::transfer::Progress;

fn pasta() -> TempDir {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("sem pasta temporária")
    };
    dir
}

fn compactar(destino: &Path, raiz: &Path, nomes: &[&str]) {
    let feito = Command::new("bsdtar")
        .arg("-a")
        .arg("-cf")
        .arg(destino)
        .arg("-C")
        .arg(raiz)
        .args(nomes)
        .status();
    assert!(feito.is_ok_and(|status| status.success()));
}

fn nomes(dir: &Path) -> Vec<String> {
    let mut nomes: Vec<String> = fs::read_dir(dir)
        .map(|items| {
            items
                .filter_map(Result::ok)
                .map(|item| item.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    nomes.sort();
    nomes
}

fn extrair(arquivo: &Path, destino: &Path) -> (std::io::Result<Extraction>, Vec<Progress>) {
    let mut vistos = Vec::new();
    let resultado = extract(arquivo, destino, &mut |progress| {
        vistos.push(progress);
        true
    });
    (resultado, vistos)
}

#[test]
fn zip_com_uma_pasta_no_topo_sai_direto_com_o_conteudo() {
    let dir = pasta();
    let origem = dir.path().join("origem");
    assert!(fs::create_dir_all(origem.join("projeto/src")).is_ok());
    assert!(fs::write(origem.join("projeto/src/main.rs"), "fn main() {}").is_ok());
    let destino = dir.path().join("destino");
    assert!(fs::create_dir(&destino).is_ok());
    let arquivo = destino.join("projeto.zip");
    compactar(&arquivo, &origem, &["projeto"]);
    let (resultado, vistos) = extrair(&arquivo, &destino);
    assert_eq!(
        resultado.ok(),
        Some(Extraction::Done(destino.join("projeto")))
    );
    assert!(destino.join("projeto/src/main.rs").is_file());
    assert_eq!(nomes(&destino), ["projeto", "projeto.zip"]);
    let ultimo = vistos.last().copied().unwrap_or_default();
    assert_eq!((ultimo.items_done, ultimo.items_total), (3, 3));
}

#[test]
fn varios_itens_no_topo_ganham_pasta_com_o_nome_do_arquivo_sem_sobrescrever() {
    let dir = pasta();
    assert!(fs::write(dir.path().join("a.txt"), "a").is_ok());
    assert!(fs::write(dir.path().join("b.txt"), "b").is_ok());
    let arquivo = dir.path().join("notas.tar.gz");
    compactar(&arquivo, dir.path(), &["a.txt", "b.txt"]);
    assert!(fs::create_dir(dir.path().join("notas")).is_ok());
    let (resultado, _) = extrair(&arquivo, dir.path());
    let Ok(Extraction::Done(saida)) = resultado else {
        panic!("a extração deveria terminar")
    };
    assert_eq!(saida, dir.path().join("notas (cópia)"));
    assert_eq!(nomes(&saida), ["a.txt", "b.txt"]);
    assert!(nomes(&dir.path().join("notas")).is_empty());
}

#[test]
fn cancelar_nao_deixa_nada_para_tras() {
    let dir = pasta();
    assert!(fs::write(dir.path().join("a.txt"), "a").is_ok());
    let arquivo = dir.path().join("x.7z");
    compactar(&arquivo, dir.path(), &["a.txt"]);
    let antes = nomes(dir.path());
    let mut chamadas = 0;
    let resultado = extract(&arquivo, dir.path(), &mut |_| {
        chamadas += 1;
        chamadas < 2
    });
    assert_eq!(chamadas, 2);
    assert_eq!(resultado.ok(), Some(Extraction::Cancelled));
    assert_eq!(nomes(dir.path()), antes);
}

#[test]
fn arquivo_corrompido_falha_sem_deixar_pasta_temporaria() {
    let dir = pasta();
    let arquivo = dir.path().join("quebrado.zip");
    assert!(fs::write(&arquivo, "isto não é um zip").is_ok());
    let (resultado, _) = extrair(&arquivo, dir.path());
    assert!(resultado.is_err());
    assert_eq!(nomes(dir.path()), ["quebrado.zip"]);
}

#[test]
fn caminho_que_sobe_de_pasta_e_recusado_sem_vazar_nada() {
    let dir = pasta();
    let origem = dir.path().join("origem");
    let alvo = dir.path().join("alvo");
    assert!(fs::create_dir_all(&origem).is_ok() && fs::create_dir_all(&alvo).is_ok());
    assert!(fs::write(origem.join("fugiu.txt"), "x").is_ok());
    let arquivo = dir.path().join("mal.tar");
    let feito = Command::new("bsdtar")
        .arg("-cf")
        .arg(&arquivo)
        .arg("-C")
        .arg(&origem)
        .arg("-s")
        .arg(",^,../,")
        .arg("fugiu.txt")
        .status();
    assert!(feito.is_ok_and(|status| status.success()));
    let (resultado, _) = extrair(&arquivo, &alvo);
    assert!(resultado.is_err());
    assert!(nomes(&alvo).is_empty());
    assert_eq!(nomes(dir.path()), ["alvo", "mal.tar", "origem"]);
}
