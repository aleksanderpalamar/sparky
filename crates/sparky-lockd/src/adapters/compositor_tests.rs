use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use sparky_ipc::codec::{decode, encode};
use sparky_ipc::{Event as Aviso, IdleHold, Reply, Request, SessionAction};
use tempfile::TempDir;
use tokio::sync::mpsc;

use crate::adapters::compositor::{Sparky, watch};
use crate::app::Event;
use crate::domain::guard::Input;
use crate::domain::inhibitions::Hold;
use crate::ports::Compositor;

fn servidor(respostas: Vec<Reply>) -> (TempDir, PathBuf, JoinHandle<Option<Request>>) {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let caminho = pasta.path().join("sparky.sock");
    let ouvinte = UnixListener::bind(&caminho).unwrap_or_else(|error| panic!("{error}"));
    let tarefa = thread::spawn(move || {
        let (conexao, _) = ouvinte.accept().ok()?;
        let mut linha = String::new();
        BufReader::new(&conexao).read_line(&mut linha).ok()?;
        let pedido = decode::<Request>(&linha).ok();
        for resposta in respostas {
            let texto = encode(&resposta).ok()?;
            (&conexao).write_all(texto.as_bytes()).ok()?;
        }
        pedido
    });
    (pasta, caminho, tarefa)
}

#[test]
fn bloquear_e_desbloquear_viram_pedidos_de_sessao_ao_sparky() {
    for (acao, esperado) in [(true, SessionAction::Lock), (false, SessionAction::Unlock)] {
        let (_pasta, caminho, tarefa) = servidor(vec![Reply::Done]);
        let sparky = Sparky::new(caminho);
        let resultado = match acao {
            true => sparky.lock(),
            false => sparky.unlock(),
        };
        assert_eq!(resultado, Ok(()));
        let pedido = tarefa.join().ok().flatten();
        assert_eq!(pedido, Some(Request::Session { action: esperado }));
    }
}

#[test]
fn a_recusa_do_sparky_volta_como_erro() {
    let recusa = Reply::Error {
        message: "não deu".to_owned(),
    };
    let (_pasta, caminho, _tarefa) = servidor(vec![recusa]);
    assert_eq!(Sparky::new(caminho).lock(), Err("não deu".to_owned()));
}

#[tokio::test]
async fn os_avisos_de_bloqueio_viram_entradas_e_o_fim_do_sparky_encerra() {
    let avisos = [Aviso::WorkspacesChanged, Aviso::Locked, Aviso::Unlocked]
        .into_iter()
        .map(|event| Reply::Event { event });
    let (_pasta, caminho, tarefa) = servidor(std::iter::once(Reply::Done).chain(avisos).collect());
    let (sender, mut receiver) = mpsc::channel(8);
    watch(caminho, sender);
    let mut recebidos = Vec::new();
    while let Some(event) = receiver.recv().await {
        recebidos.push(event);
    }
    assert_eq!(
        recebidos,
        [
            Event::Seen(Input::Locked),
            Event::Seen(Input::Unlocked),
            Event::CompositorGone
        ]
    );
    assert_eq!(tarefa.join().ok().flatten(), Some(Request::Subscribe));
}

#[tokio::test]
async fn sem_o_sparky_o_daemon_encerra() {
    let (sender, mut receiver) = mpsc::channel(8);
    watch(PathBuf::from("/nao/existe/sparky.sock"), sender);
    assert_eq!(receiver.recv().await, Some(Event::CompositorGone));
}

fn conversas(conexoes: Vec<usize>) -> (TempDir, PathBuf, JoinHandle<Vec<Request>>) {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let caminho = pasta.path().join("sparky.sock");
    let ouvinte = UnixListener::bind(&caminho).unwrap_or_else(|error| panic!("{error}"));
    let tarefa = thread::spawn(move || {
        let mut recebidos = Vec::new();
        for pedidos in conexoes {
            let Ok((conexao, _)) = ouvinte.accept() else {
                break;
            };
            atender(&conexao, pedidos, &mut recebidos);
        }
        recebidos
    });
    (pasta, caminho, tarefa)
}

fn atender(conexao: &UnixStream, pedidos: usize, recebidos: &mut Vec<Request>) {
    let mut leitor = BufReader::new(conexao);
    for _ in 0..pedidos {
        let mut linha = String::new();
        if leitor.read_line(&mut linha).unwrap_or(0) == 0 {
            return;
        }
        recebidos.extend(decode::<Request>(&linha).ok());
        let Ok(texto) = encode(&Reply::Done) else {
            return;
        };
        if (&*conexao).write_all(texto.as_bytes()).is_err() {
            return;
        }
    }
}

fn pedir_em_sequencia(caminho: PathBuf) -> Option<[Result<(), String>; 2]> {
    let (sender, receiver) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let sparky = Sparky::new(caminho);
        let _ = sender.send([sparky.idle(Hold::Held), sparky.idle(Hold::Free)]);
    });
    receiver
        .recv_timeout(std::time::Duration::from_secs(3))
        .ok()
}

#[test]
fn a_tela_acesa_vai_por_uma_so_conexao_que_fica_aberta() {
    let (_pasta, caminho, tarefa) = conversas(vec![2]);
    assert_eq!(pedir_em_sequencia(caminho), Some([Ok(()), Ok(())]));
    let esperado = [IdleHold::Held, IdleHold::Free].map(|hold| Request::Idle { hold });
    assert_eq!(tarefa.join().ok(), Some(esperado.to_vec()));
}

#[test]
fn se_a_conexao_cai_a_tela_acesa_abre_outra() {
    let (_pasta, caminho, tarefa) = conversas(vec![1, 1]);
    assert_eq!(pedir_em_sequencia(caminho), Some([Ok(()), Ok(())]));
    let esperado = [IdleHold::Held, IdleHold::Free].map(|hold| Request::Idle { hold });
    assert_eq!(tarefa.join().ok(), Some(esperado.to_vec()));
}

#[test]
fn a_tela_acesa_vira_pedido_idle_ao_sparky() {
    for (hold, esperado) in [(Hold::Held, IdleHold::Held), (Hold::Free, IdleHold::Free)] {
        let (_pasta, caminho, tarefa) = servidor(vec![Reply::Done]);
        assert_eq!(Sparky::new(caminho).idle(hold), Ok(()));
        let pedido = tarefa.join().ok().flatten();
        assert_eq!(pedido, Some(Request::Idle { hold: esperado }));
    }
}
