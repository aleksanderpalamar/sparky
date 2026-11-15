use std::path::{Path, PathBuf};

use super::{Decision, Purpose, decide};
use crate::domain::entry::Kind;

fn local() -> PathBuf {
    PathBuf::from("/home/ana/Vídeos")
}

fn arquivo(nome: &str) -> (PathBuf, Kind) {
    (local().join(nome), Kind::File)
}

fn pasta(nome: &str) -> (PathBuf, Kind) {
    (local().join(nome), Kind::Directory)
}

fn nada_existe(_: &Path) -> bool {
    false
}

fn abrir(multiple: bool, directory: bool) -> Purpose {
    Purpose::Open {
        multiple,
        directory,
    }
}

#[test]
fn abrir_arquivo_devolve_o_escolhido_e_respeita_a_escolha_unica() {
    let escolha = [arquivo("demo.mp4"), arquivo("outro.mp4")];
    assert_eq!(
        decide(&abrir(false, false), &local(), &escolha, nada_existe),
        Decision::Choose(vec![local().join("demo.mp4")])
    );
    assert_eq!(
        decide(&abrir(true, false), &local(), &escolha, nada_existe),
        Decision::Choose(vec![local().join("demo.mp4"), local().join("outro.mp4")])
    );
}

#[test]
fn abrir_arquivo_com_uma_pasta_escolhida_entra_nela_e_sem_escolha_nao_faz_nada() {
    assert_eq!(
        decide(
            &abrir(false, false),
            &local(),
            &[pasta("Férias")],
            nada_existe
        ),
        Decision::Enter(local().join("Férias"))
    );
    assert_eq!(
        decide(&abrir(false, false), &local(), &[], nada_existe),
        Decision::Nothing
    );
}

#[test]
fn escolher_pasta_devolve_a_selecionada_ou_a_atual() {
    assert_eq!(
        decide(&abrir(false, true), &local(), &[], nada_existe),
        Decision::Choose(vec![local()])
    );
    assert_eq!(
        decide(
            &abrir(false, true),
            &local(),
            &[pasta("Férias"), arquivo("a.mp4")],
            nada_existe
        ),
        Decision::Choose(vec![local().join("Férias")])
    );
}

#[test]
fn salvar_usa_o_nome_digitado_e_pede_confirmacao_se_ja_existe() {
    let salvar = |nome: &str| Purpose::Save {
        name: nome.to_owned(),
    };
    assert_eq!(
        decide(&salvar(" video.mp4 "), &local(), &[], nada_existe),
        Decision::Choose(vec![local().join("video.mp4")])
    );
    let existe = |path: &Path| path.ends_with("video.mp4");
    assert_eq!(
        decide(&salvar("video.mp4"), &local(), &[], existe),
        Decision::Confirm(vec![local().join("video.mp4")])
    );
}

#[test]
fn salvar_com_nome_invalido_entra_na_pasta_escolhida_ou_nao_faz_nada() {
    for nome in ["", "..", "a/b"] {
        let salvar = Purpose::Save {
            name: nome.to_owned(),
        };
        assert_eq!(
            decide(&salvar, &local(), &[], nada_existe),
            Decision::Nothing
        );
        assert_eq!(
            decide(&salvar, &local(), &[pasta("Férias")], nada_existe),
            Decision::Enter(local().join("Férias"))
        );
    }
}

#[test]
fn salvar_varios_coloca_os_nomes_na_pasta_escolhida() {
    let varios = Purpose::SaveFiles {
        names: vec!["a.png".to_owned(), "b.png".to_owned()],
    };
    assert_eq!(
        decide(&varios, &local(), &[pasta("Férias")], nada_existe),
        Decision::Choose(vec![
            local().join("Férias/a.png"),
            local().join("Férias/b.png")
        ])
    );
    let existe = |path: &Path| path.ends_with("b.png");
    assert!(matches!(
        decide(&varios, &local(), &[], existe),
        Decision::Confirm(_)
    ));
}

#[test]
fn o_rotulo_do_botao_perde_o_atalho_sublinhado() {
    let pedido = |accept: Option<&str>| super::Request {
        purpose: abrir(false, false),
        title: String::new(),
        accept: accept.map(str::to_owned),
        folder: None,
        filters: Vec::new(),
        filter: None,
    };
    assert_eq!(
        pedido(Some("_Open")).accept_label().as_deref(),
        Some("Open")
    );
    assert_eq!(
        pedido(Some("Save__as")).accept_label().as_deref(),
        Some("Save_as")
    );
    assert_eq!(pedido(Some("_")).accept_label(), None);
    assert_eq!(pedido(None).accept_label(), None);
}
