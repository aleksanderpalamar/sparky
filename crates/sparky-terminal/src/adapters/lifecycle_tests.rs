use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::futures::channel::mpsc;

use super::{Session, Size};
use crate::adapters::listener::Listener;
use crate::domain::grid::Cell;
use crate::domain::launch::{Launch, Program};

#[test]
fn fechar_uma_aba_libera_a_grade_e_a_thread_do_pty() {
    let (sender, _receiver) = mpsc::unbounded();
    let launch = Launch {
        program: Program::Command {
            program: "sh".into(),
            args: vec!["-c".into(), "exec sleep 30".into()],
        },
        directory: None,
    };
    let size = Size {
        columns: 20,
        lines: 4,
        cell: Cell {
            width: 8.0,
            height: 16.0,
        },
        scale: 1.0,
    };
    let Ok(session) = Session::start(&launch, size, Listener(sender)) else {
        panic!("sem PTY privado")
    };
    let lifetime = Arc::downgrade(&session.term);
    let start = Instant::now();
    drop(session);
    while lifetime.upgrade().is_some() && start.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        lifetime.upgrade().is_none(),
        "o PTY continuou vivo depois de fechar a aba"
    );
}
