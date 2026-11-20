use std::cell::RefCell;
use std::collections::HashMap;
use std::io;

use super::helper::serve;
use super::request::{Refusal, Renice};
use crate::ports::Priority;
use crate::telemetry::fake_files::FakeFiles;

const STAT: &str = "900 (rustc) R 1 900 900 0 -1 0 0 0 0 0 10 5 0 0 30 10 1 0 77 0 0";

#[derive(Default)]
struct FakePriority {
    values: RefCell<HashMap<u32, i8>>,
    stuck: bool,
}

impl Priority for FakePriority {
    fn nice(&self, task: u32) -> io::Result<i8> {
        self.values
            .borrow()
            .get(&task)
            .copied()
            .ok_or_else(|| io::Error::from_raw_os_error(libc::ESRCH))
    }

    fn set_nice(&self, task: u32, value: i8) -> io::Result<()> {
        if !self.stuck {
            self.values.borrow_mut().insert(task, value);
        }
        Ok(())
    }
}

fn files(uid: &str) -> FakeFiles {
    FakeFiles::default().with("/proc/900/stat", STAT).with(
        "/proc/900/status",
        &format!("Name:\trustc\nUid:\t{uid}\t{uid}\t{uid}\t{uid}\n"),
    )
}

fn arguments(value: i8) -> [String; 4] {
    Renice {
        task: 900,
        started: 77,
        value,
    }
    .arguments()
}

fn priority() -> FakePriority {
    let priority = FakePriority::default();
    priority.values.borrow_mut().insert(900, 10);
    priority
}

#[test]
fn devolve_o_nice_original_a_processo_do_usuario() {
    let priority = priority();
    assert_eq!(
        serve(&arguments(0), Some(1000), &files("1000"), &priority),
        Ok(())
    );
    assert_eq!(priority.values.borrow().get(&900), Some(&0));
}

#[test]
fn nao_toca_no_processo_quando_a_admissao_falha() {
    let priority = priority();
    let cases = [
        (
            arguments(-5),
            Some(1000),
            files("1000"),
            Refusal::OutOfRange,
        ),
        (arguments(0), Some(1000), files("0"), Refusal::NotOwner),
        (arguments(0), None, files("1000"), Refusal::Protected),
        (
            arguments(0),
            Some(1000),
            FakeFiles::default(),
            Refusal::Gone,
        ),
    ];
    for (arguments, caller, files, refusal) in cases {
        assert_eq!(serve(&arguments, caller, &files, &priority), Err(refusal));
    }
    assert_eq!(
        serve(&["nice".to_owned()], Some(1000), &files("1000"), &priority),
        Err(Refusal::Usage)
    );
    assert_eq!(priority.values.borrow().get(&900), Some(&10));
}

#[test]
fn confere_o_valor_depois_de_escrever() {
    let priority = FakePriority {
        stuck: true,
        ..priority()
    };
    assert_eq!(
        serve(&arguments(0), Some(1000), &files("1000"), &priority),
        Err(Refusal::Failed)
    );
}
