use std::io::Write;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::mpsc::sync_channel;

use tracing_subscriber::fmt::MakeWriter;

use super::files::{LogFile, state_directory};
use super::queue::{MAX_RECORD, Queue};

#[test]
fn caminhos_xdg_relativos_nao_redirecionam_os_logs() {
    assert_eq!(
        state_directory(Some("/state".into()), Some("/home/test".into())),
        Some("/state/sparky/logs".into())
    );
    assert_eq!(
        state_directory(Some("relativo".into()), Some("/home/test".into())),
        Some("/home/test/.local/state/sparky/logs".into())
    );
    assert_eq!(state_directory(None, Some("relativo".into())), None);
}

#[test]
fn rotacao_limita_tamanho_e_preserva_o_log_da_sessao_anterior() {
    let directory = tempfile::tempdir().unwrap();
    {
        let mut log = LogFile::open(directory.path(), 12).unwrap();
        for _ in 0..10 {
            log.write_all(b"registro\n").unwrap();
        }
        log.flush().unwrap();
    }
    let mut log = LogFile::open(directory.path(), 12).unwrap();
    log.write_all(b"nova\n").unwrap();
    log.flush().unwrap();
    assert_eq!(
        std::fs::read(directory.path().join("compositor.log.1")).unwrap(),
        b"registro\n"
    );
    for entry in std::fs::read_dir(directory.path()).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "compositor.lock" {
            continue;
        }
        assert!(entry.metadata().unwrap().len() <= 12);
    }
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 5);
}

#[test]
fn logs_sao_privados_e_uma_segunda_instancia_nao_rotaciona_o_arquivo_ativo() {
    let directory = tempfile::tempdir().unwrap();
    let log = LogFile::open(directory.path(), 1024).unwrap();
    assert!(LogFile::open(directory.path(), 1024).is_err());
    assert_eq!(
        std::fs::metadata(directory.path()).unwrap().mode() & 0o777,
        0o700
    );
    let file = std::fs::metadata(directory.path().join("compositor.log")).unwrap();
    assert_eq!(file.permissions().mode() & 0o777, 0o600);
    drop(log);
    assert!(LogFile::open(directory.path(), 1024).is_ok());
}

#[test]
fn arquivo_de_log_nao_segue_links_simbolicos() {
    let directory = tempfile::tempdir().unwrap();
    let other = directory.path().join("alvo");
    std::fs::write(&other, b"preservado").unwrap();
    symlink(&other, directory.path().join("compositor.lock")).unwrap();
    assert!(LogFile::open(directory.path(), 1024).is_err());
    assert_eq!(std::fs::read(other).unwrap(), b"preservado");
}

#[test]
fn fila_cheia_descarta_registros_sem_bloquear_o_compositor() {
    let (sender, receiver) = sync_channel(1);
    let dropped = Arc::new(AtomicU64::new(0));
    let queue = Queue::new(sender, dropped.clone());
    queue.make_writer().write_all(b"primeiro\n").unwrap();
    queue.make_writer().write_all(b"segundo\n").unwrap();
    assert_eq!(receiver.try_recv().unwrap(), b"primeiro\n");
    assert_eq!(dropped.load(Relaxed), 1);
}

#[test]
fn um_registro_grande_nao_cresce_a_fila_sem_limites() {
    let (sender, receiver) = sync_channel(1);
    let queue = Queue::new(sender, Arc::new(AtomicU64::new(0)));
    {
        let mut writer = queue.make_writer();
        writer.write_all(&vec![b'a'; MAX_RECORD * 4]).unwrap();
        writer.write_all(b"mais").unwrap();
    }
    let record = receiver.try_recv().unwrap();
    assert!(record.len() <= MAX_RECORD);
    assert!(record.ends_with(b"[truncated]\n"));
}

#[test]
fn truncar_um_registro_preserva_utf8() {
    let (sender, receiver) = sync_channel(1);
    let queue = Queue::new(sender, Arc::new(AtomicU64::new(0)));
    queue
        .make_writer()
        .write_all("á".repeat(MAX_RECORD).as_bytes())
        .unwrap();
    assert!(std::str::from_utf8(&receiver.try_recv().unwrap()).is_ok());
}
