use sparky_config::{OutputMode, OutputScale};
use sparky_ipc::{OutputInfo, SessionInfo, SessionKind};
use sparky_theme::Rgb;

use crate::domain::field::Field;
use crate::domain::status::{Reason, Status, Value};
use crate::ipc::fields::status;
use crate::ipc::port::{Problem, Snapshot};

fn info() -> SessionInfo {
    SessionInfo {
        version: "0.30.0".to_owned(),
        display: "wayland-1".to_owned(),
        session: SessionKind::Real,
        theme: Some("tokyo-night".to_owned()),
        accent: "#7aa2f7".to_owned(),
        swatches: vec!["#1a1b26".to_owned(), "#f7768e".to_owned()],
    }
}

fn monitor(name: &str, description: &str, mode: Option<OutputMode>, scale: f64) -> OutputInfo {
    OutputInfo {
        name: name.to_owned(),
        description: description.to_owned(),
        width: 1280,
        height: 720,
        mode,
        scale: OutputScale::from_factor(scale).unwrap_or(OutputScale::ONE),
        enabled: true,
        ..OutputInfo::default()
    }
}

fn retrato() -> Snapshot {
    let quatro_k = OutputMode {
        width: 2560,
        height: 1440,
        refresh: 143_973,
    };
    let full_hd = OutputMode {
        width: 1920,
        height: 1080,
        refresh: 60_000,
    };
    Snapshot {
        info: Ok(info()),
        outputs: Ok(vec![
            monitor("DP-1", "LG ULTRAGEAR", Some(quatro_k), 1.25),
            monitor("HDMI-A-1", "", Some(full_hd), 1.0),
        ]),
    }
}

#[test]
fn com_o_compositor_cada_campo_do_sparky_sai_pronto() {
    let retrato = retrato();
    assert_eq!(
        status(Field::Compositor, &retrato),
        Status::text("Sparky 0.30.0")
    );
    assert_eq!(
        status(Field::Session, &retrato),
        Status::text("Wayland · wayland-1")
    );
    assert_eq!(status(Field::Theme, &retrato), Status::text("tokyo-night"));
    assert_eq!(
        status(Field::Monitors, &retrato),
        Status::lines(vec!["DP-1 · LG ULTRAGEAR".into(), "HDMI-A-1".into()])
    );
    assert_eq!(
        status(Field::Resolution, &retrato),
        Status::lines(vec![
            "2560x1440 @ 144 Hz (×1.25)".into(),
            "1920x1080 @ 60 Hz".into()
        ])
    );
    let cores = vec![
        Rgb::new(0x7a, 0xa2, 0xf7),
        Rgb::new(0x1a, 0x1b, 0x26),
        Rgb::new(0xf7, 0x76, 0x8e),
    ];
    assert_eq!(
        status(Field::Colors, &retrato),
        Status::ready(Value::Colors(cores))
    );
}

#[test]
fn sessao_aninhada_e_tema_editado_a_mao_tem_texto_proprio() {
    let mut retrato = retrato();
    if let Ok(info) = retrato.info.as_mut() {
        info.session = SessionKind::Nested;
        info.theme = None;
    }
    assert_eq!(
        status(Field::Session, &retrato),
        Status::text("Wayland aninhado · wayland-1")
    );
    assert_eq!(
        status(Field::Theme, &retrato),
        Status::text("Personalizado")
    );
}

#[test]
fn monitor_sem_modo_usa_o_tamanho_logico() {
    let retrato = Snapshot {
        info: Ok(info()),
        outputs: Ok(vec![monitor("WL-1", "", None, 1.0)]),
    };
    assert_eq!(
        status(Field::Resolution, &retrato),
        Status::lines(vec!["1280x720".into()])
    );
}

#[test]
fn cada_problema_vira_um_estado_explicito() {
    let casos = [
        (
            Problem::NoCompositor,
            Status::unavailable(Reason::NoCompositor),
        ),
        (
            Problem::Unsupported,
            Status::unavailable(Reason::Unsupported),
        ),
        (Problem::TimedOut, Status::TimedOut),
        (Problem::Failed("quebrou".into()), Status::failed("quebrou")),
    ];
    for (problema, esperado) in casos {
        let retrato = Snapshot {
            info: Err(problema.clone()),
            outputs: Err(problema),
        };
        for field in [
            Field::Compositor,
            Field::Session,
            Field::Monitors,
            Field::Theme,
            Field::Colors,
        ] {
            assert_eq!(status(field, &retrato), esperado, "{field:?}");
        }
    }
}

#[test]
fn compositor_antigo_ainda_informa_os_monitores() {
    let mut retrato = retrato();
    retrato.info = Err(Problem::Unsupported);
    assert_eq!(
        status(Field::Compositor, &retrato),
        Status::unavailable(Reason::Unsupported)
    );
    assert!(matches!(
        status(Field::Monitors, &retrato),
        Status::Ready { .. }
    ));
}

#[test]
fn sem_monitores_e_cores_invalidas_ficam_indisponiveis() {
    let mut retrato = retrato();
    retrato.outputs = Ok(Vec::new());
    if let Ok(info) = retrato.info.as_mut() {
        info.accent = String::new();
    }
    assert_eq!(
        status(Field::Monitors, &retrato),
        Status::unavailable(Reason::Missing)
    );
    assert_eq!(
        status(Field::Colors, &retrato),
        Status::unavailable(Reason::Missing)
    );
}
