use serde_json::json;
use sparky_theme::Rgb;

use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::{Reason, Status, Value};

#[test]
fn o_json_detalha_cada_estado() {
    let report = Report {
        entries: vec![
            Entry::new(Field::Kernel, Status::text("6.17.1-arch1-1")),
            Entry::new(Field::Gpu, Status::unavailable(Reason::Missing)),
            Entry::new(Field::Packages, Status::TimedOut),
            Entry::new(Field::Disk, Status::failed("permissão negada")),
            Entry::new(Field::Monitors, Status::lines(vec!["DP-1".into()])),
            Entry::new(
                Field::Colors,
                Status::ready(Value::Colors(vec![Rgb::new(0x7a, 0xa2, 0xf7)])),
            ),
        ],
    };
    let esperado = json!({"entries": [
        {"field": "kernel", "status": "ready", "value": "6.17.1-arch1-1"},
        {"field": "gpu", "status": "unavailable", "reason": "missing"},
        {"field": "packages", "status": "timed_out"},
        {"field": "disk", "status": "failed", "error": "permissão negada"},
        {"field": "monitors", "status": "ready", "value": ["DP-1"]},
        {"field": "colors", "status": "ready", "value": ["#7aa2f7"]},
    ]});
    assert_eq!(serde_json::to_value(&report).ok(), Some(esperado));
}

#[test]
fn o_relatorio_acha_o_estado_de_um_campo() {
    let report = Report {
        entries: vec![Entry::new(Field::Os, Status::text("Arch Linux"))],
    };
    assert_eq!(
        report.status(Field::Os).and_then(Status::value),
        Some(&Value::Text("Arch Linux".into()))
    );
    assert_eq!(report.status(Field::Cpu), None);
}
