use std::cell::RefCell;
use std::process::{Child, Command, Stdio};

use super::executor::LinuxExecutor;
use super::files::LinuxFiles;
use crate::domain::{Adjustment, Change, Identity, IoClass};
use crate::ports::{Elevator, ExecError, Executor};
use crate::safety::{Outcome, apply, restore};

fn child() -> Option<(Child, Identity)> {
    let child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .ok()?;
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", child.id())).ok()?;
    let started = sparky_hardware::process::parse_stat(&stat)?.started;
    let identity = Identity {
        pid: child.id(),
        started,
    };
    Some((child, identity))
}

#[derive(Default)]
struct Recorder {
    calls: RefCell<Vec<(u32, i8)>>,
}

impl Elevator for Recorder {
    fn renice(&self, task: u32, value: i8) -> Result<(), ExecError> {
        self.calls.borrow_mut().push((task, value));
        Ok(())
    }
}

fn may_raise_priority() -> bool {
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let read = unsafe { libc::getrlimit(libc::RLIMIT_NICE, &mut limit) };
    read != 0 || limit.rlim_cur > 20
}

fn finish(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn afinidade_e_prioridade_de_io_sao_aplicadas_e_restauradas_de_verdade() {
    let Some((process, target)) = child() else {
        return;
    };
    let executor = LinuxExecutor::new(LinuxFiles, Recorder::default());
    for change in [Change::IoPriority(IoClass::Idle), Change::Affinity(vec![0])] {
        let adjustment = Adjustment { target, change };
        let applied = apply(&executor, &adjustment);
        assert!(applied.is_ok(), "{adjustment:?}: {applied:?}");
        if let Ok(applied) = applied {
            assert_eq!(
                restore(&executor, &applied),
                Outcome::Restored,
                "{adjustment:?}"
            );
        }
    }
    finish(process);
}

#[test]
fn pid_reaproveitado_nao_e_confundido_com_o_processo_original() {
    let Some((process, target)) = child() else {
        return;
    };
    let executor = LinuxExecutor::new(LinuxFiles, Recorder::default());
    let impostor = Identity {
        started: target.started + 1,
        ..target
    };
    assert_eq!(executor.tasks(impostor), Err(ExecError::Gone));
    assert!(
        executor
            .tasks(target)
            .is_ok_and(|tasks| tasks.contains(&target.pid))
    );
    finish(process);
}

#[test]
fn sem_privilegio_devolver_o_nice_passa_pelo_helper() {
    let Some((process, target)) = child() else {
        return;
    };
    if may_raise_priority() {
        return finish(process);
    }
    let recorder = Recorder::default();
    let executor = LinuxExecutor::new(LinuxFiles, &recorder);
    assert_eq!(executor.write(target.pid, &Change::Nice(10)), Ok(()));
    assert!(recorder.calls.borrow().is_empty());
    assert_eq!(executor.write(target.pid, &Change::Nice(0)), Ok(()));
    assert_eq!(*recorder.calls.borrow(), vec![(target.pid, 0)]);
    finish(process);
}
