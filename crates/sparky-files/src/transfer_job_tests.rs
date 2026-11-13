use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::domain::transfer::{Action, Ending, Progress, Step};
use crate::domain::undo::TrashEntry;
use crate::ports::Transfer;
use crate::transfer_job::execute;

struct Simulada {
    falha: Option<PathBuf>,
    cancelar_em: Option<PathBuf>,
    cancel: Option<&'static AtomicBool>,
    feitos: Mutex<Vec<PathBuf>>,
}

impl Simulada {
    fn nova() -> Self {
        Self {
            falha: None,
            cancelar_em: None,
            cancel: None,
            feitos: Mutex::new(Vec::new()),
        }
    }
}

impl Transfer for Simulada {
    fn size(&self, _path: &Path) -> u64 {
        10
    }

    fn run(
        &self,
        step: &Step,
        advance: &mut dyn FnMut(u64) -> bool,
    ) -> io::Result<Option<TrashEntry>> {
        if self.falha.as_ref() == Some(&step.source) {
            return Err(io::Error::other("disco cheio"));
        }
        if self.cancelar_em.as_ref() == Some(&step.source) {
            self.cancel.unwrap().store(true, Ordering::SeqCst);
        }
        if !advance(10) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        self.feitos.lock().unwrap().push(step.source.clone());
        Ok(None)
    }
}

fn passos(nomes: &[&str]) -> Vec<Step> {
    nomes
        .iter()
        .map(|nome| Step {
            source: PathBuf::from(format!("/o/{nome}")),
            target: PathBuf::from(format!("/d/{nome}")),
            action: Action::Copy,
            replace: false,
        })
        .collect()
}

#[test]
fn executa_todos_os_passos_e_informa_o_progresso_final() {
    let transfer = Simulada::nova();
    let mut relatos = Vec::new();
    let outcome = execute(
        passos(&["a", "b"]),
        1,
        &transfer,
        &AtomicBool::new(false),
        &mut |p| relatos.push(p),
    );
    assert_eq!(outcome.ending, Ending::Finished);
    assert_eq!(outcome.completed.len(), 2);
    assert_eq!(outcome.skipped, 1);
    assert_eq!(relatos.first().map(|p: &Progress| p.bytes_total), Some(20));
    let ultimo = relatos.last().copied().unwrap();
    assert_eq!(
        (ultimo.items_done, ultimo.bytes_done, ultimo.percent()),
        (2, 20, 100)
    );
}

#[test]
fn cancelar_no_meio_para_e_guarda_o_que_ja_terminou() {
    static CANCEL: AtomicBool = AtomicBool::new(false);
    let transfer = Simulada {
        cancelar_em: Some(PathBuf::from("/o/b")),
        cancel: Some(&CANCEL),
        ..Simulada::nova()
    };
    let outcome = execute(passos(&["a", "b", "c"]), 0, &transfer, &CANCEL, &mut |_| {});
    assert_eq!(outcome.ending, Ending::Cancelled);
    assert_eq!(outcome.completed.len(), 1);
    assert_eq!(*transfer.feitos.lock().unwrap(), [PathBuf::from("/o/a")]);
}

#[test]
fn falha_informa_o_item_e_o_erro() {
    let transfer = Simulada {
        falha: Some(PathBuf::from("/o/b")),
        ..Simulada::nova()
    };
    let outcome = execute(
        passos(&["a", "b", "c"]),
        0,
        &transfer,
        &AtomicBool::new(false),
        &mut |_| {},
    );
    assert_eq!(
        outcome.ending,
        Ending::Failed {
            name: "b".to_owned(),
            error: "disco cheio".to_owned()
        }
    );
    assert_eq!(outcome.completed.len(), 1);
}
