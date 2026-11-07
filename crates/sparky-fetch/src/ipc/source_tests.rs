use std::sync::atomic::{AtomicUsize, Ordering};

use crate::domain::field::Field;
use crate::domain::status::{Reason, Status};
use crate::ipc::port::{Compositor, Problem, Snapshot};
use crate::ipc::source::SparkySource;
use crate::sources::Source;

struct Ausente(AtomicUsize);

impl Compositor for Ausente {
    fn snapshot(&self) -> Snapshot {
        self.0.fetch_add(1, Ordering::SeqCst);
        Snapshot::absent(Problem::NoCompositor)
    }
}

#[test]
fn so_devolve_os_campos_pedidos_com_uma_consulta() {
    let fonte = SparkySource::new(Ausente(AtomicUsize::new(0)));
    let campos = fonte.collect(&[Field::Os, Field::Theme, Field::Monitors]);
    let sem_compositor = Status::unavailable(Reason::NoCompositor);
    assert_eq!(
        campos,
        vec![
            (Field::Monitors, sem_compositor.clone()),
            (Field::Theme, sem_compositor)
        ]
    );
    assert_eq!(fonte.fields().len(), 6);
}
