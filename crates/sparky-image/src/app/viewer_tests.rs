use std::path::Path;

use super::keys::Command;
use super::message::Message;
use super::state::{Shown, Viewer};
use super::test_support::{abrir, concluir};
use super::update::update;
use crate::domain::album::Step;
use crate::domain::viewport::Direction;

fn comando(viewer: &mut Viewer, command: Command) {
    let task = update(viewer, Message::Command(command));
    concluir(viewer, task);
}

fn atual(viewer: &Viewer) -> &Path {
    viewer
        .album
        .as_ref()
        .map(|album| album.current())
        .unwrap_or(Path::new(""))
}

#[test]
fn abrir_arquivo_carrega_a_imagem_e_conhece_a_pasta() {
    let (mut viewer, task) = abrir(vec!["a.png", "b.png", "c.png"], Some("/fotos/b.png"));
    assert!(matches!(viewer.shown, Shown::Loading));
    concluir(&mut viewer, task);
    assert!(matches!(&viewer.shown, Shown::Ready(ready) if ready.width == 200));
    assert_eq!(
        viewer.album.as_ref().map(|album| album.position()),
        Some((2, 3))
    );
    assert_eq!(super::title(&viewer), "b.png — Imagens");
}

#[test]
fn setas_trocam_de_imagem_quando_ela_cabe_na_janela() {
    let (mut viewer, task) = abrir(vec!["a.png", "b.png"], Some("/fotos/a.png"));
    concluir(&mut viewer, task);
    comando(&mut viewer, Command::Arrow(Direction::Right));
    assert_eq!(atual(&viewer), Path::new("/fotos/b.png"));
    assert!(matches!(viewer.shown, Shown::Ready(_)));
    comando(&mut viewer, Command::Arrow(Direction::Left));
    assert_eq!(atual(&viewer), Path::new("/fotos/a.png"));
}

#[test]
fn com_zoom_as_setas_deslocam_e_espaco_continua_trocando() {
    let (mut viewer, task) = abrir(vec!["grande.png", "z.png"], Some("/fotos/grande.png"));
    concluir(&mut viewer, task);
    comando(&mut viewer, Command::ActualSize);
    comando(&mut viewer, Command::Arrow(Direction::Right));
    assert_eq!(atual(&viewer), Path::new("/fotos/grande.png"));
    let Shown::Ready(ready) = &viewer.shown else {
        panic!("imagem não carregada");
    };
    assert_eq!(ready.viewport.placement(viewer.area()).x, -580.0);
    comando(&mut viewer, Command::Step(Step::Next));
    assert_eq!(atual(&viewer), Path::new("/fotos/z.png"));
}

#[test]
fn resultado_de_imagem_ja_deixada_para_tras_e_descartado() {
    let (mut viewer, _) = abrir(vec!["a.png", "b.png", "c.png"], Some("/fotos/a.png"));
    let _ = update(&mut viewer, Message::Command(Command::Step(Step::Next)));
    let task = update(&mut viewer, Message::Command(Command::Step(Step::Next)));
    let antigo = super::test_support::Decodificador;
    let picture = crate::ports::Decoder::decode(&antigo, Path::new("/fotos/b.png"));
    let _ = update(&mut viewer, Message::Loaded("/fotos/b.png".into(), picture));
    assert!(matches!(viewer.shown, Shown::Loading));
    concluir(&mut viewer, task);
    assert_eq!(atual(&viewer), Path::new("/fotos/c.png"));
    assert!(matches!(viewer.shown, Shown::Ready(_)));
}

#[test]
fn falha_ao_decodificar_aparece_na_tela() {
    let (mut viewer, task) = abrir(vec!["quebrada.png"], Some("/fotos/quebrada.png"));
    concluir(&mut viewer, task);
    assert!(
        matches!(&viewer.shown, Shown::Failed(text) if text == "Imagem corrompida ou incompleta")
    );
}

#[test]
fn sem_argumento_ou_pasta_sem_imagens_mostra_estado_vazio() {
    let (viewer, _) = abrir(vec!["a.png"], None);
    assert!(matches!(viewer.shown, Shown::Empty(_)));
    assert_eq!(super::title(&viewer), "Imagens");
    let dir = tempfile::tempdir().unwrap();
    let (vazia, _) = abrir(Vec::new(), dir.path().to_str());
    assert!(matches!(vazia.shown, Shown::Empty(text) if text == "Nenhuma imagem nesta pasta"));
}

#[test]
fn pasta_aberta_comeca_pela_primeira_imagem() {
    let dir = tempfile::tempdir().unwrap();
    let (mut viewer, task) = abrir(vec!["b.png", "a.png"], dir.path().to_str());
    concluir(&mut viewer, task);
    assert_eq!(atual(&viewer), dir.path().join("a.png"));
    assert!(matches!(viewer.shown, Shown::Ready(_)));
}

#[test]
fn imagem_unica_nao_e_recarregada_ao_navegar() {
    let (mut viewer, task) = abrir(vec!["a.png"], Some("/fotos/a.png"));
    concluir(&mut viewer, task);
    let task = update(&mut viewer, Message::Command(Command::Step(Step::Next)));
    assert!(iced_runtime::task::into_stream(task).is_none());
    assert!(matches!(viewer.shown, Shown::Ready(_)));
}

#[test]
fn barra_de_status_mostra_nome_posicao_tamanho_e_zoom() {
    let (mut viewer, task) = abrir(vec!["a.png", "grande.png"], Some("/fotos/grande.png"));
    assert_eq!(crate::view::status::summary(&viewer), "grande.png · 2/2");
    concluir(&mut viewer, task);
    assert_eq!(
        crate::view::status::summary(&viewer),
        "grande.png · 2/2 · 2000 × 1000 · 50%"
    );
}
