use super::fake_files::FakeFiles;
use super::processes::scan;
use crate::context::Process;

fn stat(pid: u32, name: &str, parent: u32) -> String {
    format!("{pid} ({name}) S {parent} 0 0 0 -1 0 0 0 0 0 1 1 0 0 20 0 1 0 100 0")
}

#[test]
fn le_processos_e_ignora_threads_do_kernel_e_entradas_nao_numericas() {
    let files = FakeFiles::default()
        .with("/proc/1/stat", &stat(1, "systemd", 0))
        .with("/proc/2/stat", &stat(2, "kthreadd", 0))
        .with("/proc/33/stat", &stat(33, "kworker/0:1", 2))
        .with("/proc/900/stat", &stat(900, "obs", 1))
        .with("/proc/self/stat", &stat(77, "fantasma", 1))
        .with("/proc/meminfo", "MemTotal: 1 kB\n");
    let found = scan(&files).unwrap_or_default();
    assert_eq!(
        found,
        vec![
            Process {
                pid: 1,
                parent: 0,
                name: "systemd".to_owned()
            },
            Process {
                pid: 900,
                parent: 1,
                name: "obs".to_owned()
            },
        ]
    );
}

#[test]
fn processo_que_sumiu_no_meio_da_leitura_e_ignorado() {
    let files = FakeFiles::default()
        .with("/proc/10/status", "Name: sumiu\n")
        .with("/proc/11/stat", &stat(11, "zsh", 1));
    assert_eq!(scan(&files).map(|found| found.len()), Some(1));
}

#[test]
fn sem_proc_nao_ha_lista() {
    assert_eq!(scan(&FakeFiles::default()), None);
}

#[test]
fn le_so_os_processos_novos_e_esquece_os_que_sairam() {
    use super::processes::{ProcessTable, Sweep};
    use std::time::{Duration, Instant};
    let start = Instant::now();
    let mut table = ProcessTable::default();
    let before = FakeFiles::default()
        .with("/proc/1/stat", &stat(1, "systemd", 0))
        .with("/proc/5/stat", &stat(5, "steam", 1));
    assert_eq!(
        table
            .update(&before, start, Sweep::Routine)
            .map(|found| found.len()),
        Some(2)
    );
    let renamed = FakeFiles::default()
        .with("/proc/1/stat", &stat(1, "init", 0))
        .with("/proc/9/stat", &stat(9, "obs", 1));
    let after = table
        .update(&renamed, start + Duration::from_secs(5), Sweep::Routine)
        .unwrap_or_default();
    let names: Vec<&str> = after.iter().map(|process| process.name.as_str()).collect();
    assert_eq!(names, vec!["systemd", "obs"]);
    let renewed = (2..=9).any(|tick| {
        table
            .update(
                &renamed,
                start + Duration::from_secs(5 * tick),
                Sweep::Routine,
            )
            .unwrap_or_default()
            .first()
            .is_some_and(|process| process.name == "init")
    });
    assert!(renewed);
}
