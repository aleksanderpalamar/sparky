use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::domain::field::Field;
use crate::domain::status::{Reason, Status};
use crate::run::{Limits, collect};
use crate::sources::Source;

struct Fixa(&'static [Field], Duration);

impl Source for Fixa {
    fn fields(&self) -> &'static [Field] {
        self.0
    }

    fn collect(&self, wanted: &[Field]) -> Vec<(Field, Status)> {
        thread::sleep(self.1);
        self.0
            .iter()
            .filter(|field| wanted.contains(field))
            .map(|field| (*field, Status::text(field.key())))
            .collect()
    }
}

struct Contada(Arc<AtomicUsize>, Arc<AtomicUsize>);

impl Source for Contada {
    fn fields(&self) -> &'static [Field] {
        &[Field::Os]
    }

    fn collect(&self, _: &[Field]) -> Vec<(Field, Status)> {
        let agora = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        self.1.fetch_max(agora, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(30));
        self.0.fetch_sub(1, Ordering::SeqCst);
        vec![(Field::Os, Status::text("ok"))]
    }
}

fn limites(workers: usize, ms: u64) -> Limits {
    Limits {
        workers,
        timeout: Duration::from_millis(ms),
    }
}

#[test]
fn o_relatorio_segue_a_ordem_pedida_e_marca_o_que_ninguem_coleta() {
    let fontes: Vec<Arc<dyn Source>> = vec![
        Arc::new(Fixa(&[Field::Os, Field::Kernel], Duration::ZERO)),
        Arc::new(Fixa(&[Field::Cpu], Duration::ZERO)),
    ];
    let pedidos = [Field::Cpu, Field::Gpu, Field::Os];
    let report = collect(fontes, &pedidos, limites(2, 500));
    let campos: Vec<Field> = report.entries.iter().map(|entry| entry.field).collect();
    assert_eq!(campos, pedidos);
    assert_eq!(report.status(Field::Cpu), Some(&Status::text("cpu")));
    assert_eq!(
        report.status(Field::Gpu),
        Some(&Status::unavailable(Reason::Uncollected))
    );
    assert_eq!(report.status(Field::Kernel), None);
}

#[test]
fn uma_fonte_lenta_vira_tempo_esgotado_sem_segurar_as_outras() {
    let fontes: Vec<Arc<dyn Source>> = vec![
        Arc::new(Fixa(&[Field::Packages], Duration::from_secs(5))),
        Arc::new(Fixa(&[Field::Os], Duration::ZERO)),
    ];
    let inicio = Instant::now();
    let report = collect(fontes, &[Field::Packages, Field::Os], limites(2, 100));
    assert!(inicio.elapsed() < Duration::from_secs(1));
    assert_eq!(report.status(Field::Packages), Some(&Status::TimedOut));
    assert_eq!(report.status(Field::Os), Some(&Status::text("os")));
}

#[test]
fn termina_assim_que_todas_as_fontes_respondem() {
    let fontes: Vec<Arc<dyn Source>> = vec![Arc::new(Fixa(&[Field::Os], Duration::ZERO))];
    let inicio = Instant::now();
    let _ = collect(fontes, &[Field::Os], limites(2, 2_000));
    assert!(inicio.elapsed() < Duration::from_millis(500));
}

#[test]
fn nunca_roda_mais_fontes_ao_mesmo_tempo_que_o_limite() {
    let ativas = Arc::new(AtomicUsize::new(0));
    let pico = Arc::new(AtomicUsize::new(0));
    let fontes: Vec<Arc<dyn Source>> = (0..8)
        .map(|_| Arc::new(Contada(Arc::clone(&ativas), Arc::clone(&pico))) as Arc<dyn Source>)
        .collect();
    let _ = collect(fontes, &[Field::Os], limites(3, 2_000));
    assert!(pico.load(Ordering::SeqCst) <= 3);
    assert!(pico.load(Ordering::SeqCst) >= 2);
}

#[test]
fn fonte_sem_campo_pedido_nem_roda() {
    let fontes: Vec<Arc<dyn Source>> = vec![Arc::new(Fixa(&[Field::Gpu], Duration::from_secs(5)))];
    let inicio = Instant::now();
    let report = collect(fontes, &[Field::Os], limites(1, 1_000));
    assert!(inicio.elapsed() < Duration::from_millis(500));
    assert_eq!(
        report.status(Field::Os),
        Some(&Status::unavailable(Reason::Uncollected))
    );
}
