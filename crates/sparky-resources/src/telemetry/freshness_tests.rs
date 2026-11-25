use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::fake_files::FakeFiles;
use super::processes::{ProcessTable, Sweep};
use crate::ports::Files;

struct Spy {
    files: RefCell<FakeFiles>,
    reads: RefCell<HashMap<PathBuf, usize>>,
}

impl Spy {
    fn new(files: FakeFiles) -> Self {
        Self {
            files: RefCell::new(files),
            reads: RefCell::new(HashMap::new()),
        }
    }

    fn reads(&self, pid: u32) -> usize {
        let path = PathBuf::from(format!("/proc/{pid}/stat"));
        self.reads.borrow().get(&path).copied().unwrap_or(0)
    }

    fn swap(&self, files: FakeFiles) {
        *self.files.borrow_mut() = files;
    }
}

impl Files for &Spy {
    fn read(&self, path: &Path) -> io::Result<String> {
        *self.reads.borrow_mut().entry(path.to_owned()).or_default() += 1;
        self.files.borrow().read(path)
    }

    fn entries(&self, directory: &Path) -> io::Result<Vec<String>> {
        self.files.borrow().entries(directory)
    }
}

fn stat(pid: u32, name: &str, parent: u32, started: u64) -> String {
    format!("{pid} ({name}) S {parent} 0 0 0 -1 0 0 0 0 0 1 1 0 0 20 0 1 0 {started} 0")
}

fn machine(changed: Option<(&str, u32, u64)>) -> FakeFiles {
    (1..=16).fold(FakeFiles::default(), |files, pid| {
        let (name, parent, started) = match (pid, changed) {
            (9, Some(change)) => change,
            _ => ("app", 1, 100),
        };
        files.with(
            &format!("/proc/{pid}/stat"),
            &stat(pid, name, parent, started),
        )
    })
}

fn find(
    table: &mut ProcessTable,
    spy: &Spy,
    at: Instant,
    sweep: Sweep,
    pid: u32,
) -> Option<(String, u32)> {
    table
        .update(&spy, at, sweep)?
        .into_iter()
        .find(|process| process.pid == pid)
        .map(|process| (process.name, process.parent))
}

#[test]
fn falha_de_leitura_nao_fica_guardada_e_e_relida_no_ciclo_seguinte() {
    let start = Instant::now();
    let spy = Spy::new(machine(None).broken("/proc/9/stat", ErrorKind::Other));
    let mut table = ProcessTable::default();
    assert_eq!(find(&mut table, &spy, start, Sweep::Routine, 9), None);
    assert_eq!(spy.reads(9), 1);
    spy.swap(machine(None));
    let next = start + Duration::from_secs(2);
    assert_eq!(
        find(&mut table, &spy, next, Sweep::Routine, 9),
        Some(("app".to_owned(), 1))
    );
    assert_eq!(spy.reads(9), 2);
}

#[test]
fn nome_e_pai_novos_aparecem_em_ate_oito_ciclos() {
    let start = Instant::now();
    let spy = Spy::new(machine(None));
    let mut table = ProcessTable::default();
    find(&mut table, &spy, start, Sweep::Routine, 9);
    spy.swap(machine(Some(("jogo", 4, 100))));
    let seen = (1..=8).find_map(|tick| {
        let at = start + Duration::from_secs(2 * tick);
        find(&mut table, &spy, at, Sweep::Routine, 9).filter(|found| found.0 == "jogo")
    });
    assert_eq!(seen, Some(("jogo".to_owned(), 4)));
}

#[test]
fn pid_reaproveitado_e_relido_na_hora_da_decisao() {
    let start = Instant::now();
    let spy = Spy::new(machine(None));
    let mut table = ProcessTable::default();
    find(&mut table, &spy, start, Sweep::Routine, 9);
    spy.swap(machine(Some(("outro", 1, 900))));
    let at = start + Duration::from_secs(2);
    assert_eq!(
        find(&mut table, &spy, at, Sweep::Decision, 9),
        Some(("outro".to_owned(), 1))
    );
}

#[test]
fn decisao_rele_todos_no_maximo_uma_vez_por_segundo() {
    let start = Instant::now();
    let spy = Spy::new(machine(None));
    let mut table = ProcessTable::default();
    find(&mut table, &spy, start, Sweep::Routine, 9);
    let total = |spy: &Spy| (1..=16).map(|pid| spy.reads(pid)).sum::<usize>();
    let before = total(&spy);
    find(
        &mut table,
        &spy,
        start + Duration::from_millis(1100),
        Sweep::Decision,
        9,
    );
    assert_eq!(total(&spy) - before, 16);
    let again = total(&spy);
    find(
        &mut table,
        &spy,
        start + Duration::from_millis(1350),
        Sweep::Decision,
        9,
    );
    assert!(total(&spy) - again < 16);
}
