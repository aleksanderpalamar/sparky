use super::table::{Column, Order, Sort, sort};
use super::usage::{ProcessRow, Usage};

fn linha(name: &str, cpu: f32, memory: u64, gpu: Option<f32>) -> ProcessRow {
    ProcessRow {
        pid: memory as u32,
        ppid: 1,
        name: name.to_owned(),
        program: name.to_owned(),
        user: Some(1000),
        usage: Usage {
            cpu,
            memory,
            gpu,
            ..Usage::default()
        },
    }
}

fn nomes(rows: &[ProcessRow]) -> Vec<&str> {
    rows.iter().map(|row| row.name.as_str()).collect()
}

#[test]
fn padrao_e_cpu_decrescente_com_nome_desempatando() {
    let mut rows = vec![
        linha("b", 1.0, 1, None),
        linha("c", 5.0, 2, None),
        linha("a", 1.0, 3, None),
    ];
    sort(&mut rows, Sort::default());
    assert_eq!(nomes(&rows), ["c", "a", "b"]);
}

#[test]
fn trocar_de_coluna_usa_a_ordem_natural_e_repetir_inverte() {
    let sort_ = Sort::default().by(Column::Name);
    assert_eq!(
        sort_,
        Sort {
            column: Column::Name,
            order: Order::Ascending
        }
    );
    assert_eq!(sort_.by(Column::Name).order, Order::Descending);
    assert_eq!(sort_.by(Column::Memory).order, Order::Descending);
}

#[test]
fn memoria_e_gpu_ausente_ficam_por_ultimo() {
    let mut rows = vec![
        linha("x", 0.0, 10, None),
        linha("y", 0.0, 30, Some(2.0)),
        linha("z", 0.0, 20, Some(0.0)),
    ];
    sort(&mut rows, Sort::default().by(Column::Memory));
    assert_eq!(nomes(&rows), ["y", "z", "x"]);
    sort(&mut rows, Sort::default().by(Column::Gpu));
    assert_eq!(nomes(&rows), ["y", "z", "x"]);
    sort(&mut rows, Sort::default().by(Column::Name).by(Column::Name));
    assert_eq!(nomes(&rows), ["z", "y", "x"]);
}

#[test]
fn maior_consumidor_de_cada_recurso() {
    let rows = vec![linha("a", 1.0, 900, None), linha("b", 7.0, 100, Some(3.0))];
    assert_eq!(
        super::table::top(&rows, Column::Cpu).map(|row| row.name.as_str()),
        Some("b")
    );
    assert_eq!(
        super::table::top(&rows, Column::Memory).map(|row| row.name.as_str()),
        Some("a")
    );
    assert_eq!(
        super::table::top(&rows, Column::Gpu).map(|row| row.name.as_str()),
        Some("b")
    );
    assert!(super::table::top::<super::usage::ProcessRow>(&[], Column::Cpu).is_none());
}
