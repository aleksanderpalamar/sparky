use std::process::{Child, Command, Stdio};

use super::executor::LinuxExecutor;
use super::files::LinuxFiles;
use crate::domain::{Adjustment, Change, Identity, IoClass};
use crate::ports::{ExecError, Executor};
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

fn finish(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn afinidade_e_prioridade_de_io_sao_aplicadas_e_restauradas_de_verdade() {
    let Some((process, target)) = child() else {
        return;
    };
    let executor = LinuxExecutor::new(LinuxFiles);
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
    let executor = LinuxExecutor::new(LinuxFiles);
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
