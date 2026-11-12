use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::watch::watch;

struct Vigia {
    _dir: tempfile::TempDir,
    pasta: PathBuf,
    vivo: Arc<AtomicBool>,
    avisos: mpsc::Receiver<()>,
    linha: thread::JoinHandle<std::io::Result<()>>,
}

impl Vigia {
    fn nova() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let pasta = dir.path().to_path_buf();
        fs::write(pasta.join("nota.txt"), "a").unwrap();
        let vivo = Arc::new(AtomicBool::new(true));
        let (envio, avisos) = mpsc::channel();
        let (alvo, ativo) = (pasta.clone(), vivo.clone());
        let linha = thread::spawn(move || {
            watch(
                &alvo,
                || ativo.load(Ordering::SeqCst),
                || envio.send(()).is_ok(),
            )
        });
        thread::sleep(Duration::from_millis(100));
        Self {
            _dir: dir,
            pasta,
            vivo,
            avisos,
            linha,
        }
    }

    fn avisou(&self) -> bool {
        self.avisos.recv_timeout(Duration::from_secs(3)).is_ok()
    }

    fn encerrar(self) {
        self.vivo.store(false, Ordering::SeqCst);
        assert!(self.linha.join().unwrap().is_ok());
    }
}

#[test]
fn arquivo_criado_na_pasta_avisa_uma_vez_por_rajada() {
    let vigia = Vigia::nova();
    for indice in 0..50 {
        fs::write(vigia.pasta.join(format!("{indice}.txt")), "x").unwrap();
    }
    assert!(vigia.avisou());
    assert!(
        vigia
            .avisos
            .recv_timeout(Duration::from_millis(600))
            .is_err()
    );
    vigia.encerrar();
}

#[test]
fn arquivo_alterado_no_lugar_tambem_avisa() {
    let vigia = Vigia::nova();
    fs::write(vigia.pasta.join("nota.txt"), "conteudo novo").unwrap();
    assert!(vigia.avisou());
    vigia.encerrar();
}

#[test]
fn vigia_termina_quando_ninguem_mais_escuta() {
    let vigia = Vigia::nova();
    vigia.encerrar();
}

#[test]
fn pasta_inexistente_devolve_erro() {
    let resultado = watch(&PathBuf::from("/nao/existe/sparky"), || true, || true);
    assert!(resultado.is_err());
}
