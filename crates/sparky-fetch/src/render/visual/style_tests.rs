use sparky_theme::{Palette, Rgb};

use crate::config::settings::Colors;
use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::Status;
use crate::render::view::View;
use crate::render::visual::fixture::{ACCENT, report, view};
use crate::render::visual::segment::{Line, Segment, Tone};
use crate::render::visual::style::{Inks, Paint};

const ROSA: Rgb = Rgb::new(0xff, 0x00, 0x80);
const VERDE: Rgb = Rgb::new(0x00, 0xff, 0x00);

fn tintas(report: &Report, colors: Colors) -> Option<Inks> {
    let visao = View {
        colors,
        ..view(80, true)
    };
    match Paint::resolve(report, &visao) {
        Paint::TrueColor(inks) => Some(inks),
        Paint::Plain => None,
    }
}

#[test]
fn sem_cor_nao_ha_tintas() {
    assert_eq!(Paint::resolve(&report(), &view(80, false)), Paint::Plain);
}

#[test]
fn o_destaque_vem_da_primeira_cor_do_tema() {
    let esperado = Inks {
        accent: ACCENT,
        label: ACCENT,
        value: None,
    };
    assert_eq!(tintas(&report(), Colors::default()), Some(esperado));
}

#[test]
fn sem_paleta_o_destaque_e_o_do_tokyo_night() {
    let sem_cores = Report {
        entries: vec![Entry::new(Field::Colors, Status::TimedOut)],
    };
    let accent = Palette::TOKYO_NIGHT.accent;
    let esperado = Inks {
        accent,
        label: accent,
        value: None,
    };
    assert_eq!(tintas(&sem_cores, Colors::default()), Some(esperado));
}

#[test]
fn as_cores_pedidas_vencem_o_tema() {
    let pedidas = Colors {
        accent: Some(ROSA),
        label: None,
        value: Some(VERDE),
    };
    let esperado = Inks {
        accent: ROSA,
        label: ROSA,
        value: Some(VERDE),
    };
    assert_eq!(tintas(&report(), pedidas), Some(esperado));
    let rotulo = Colors {
        label: Some(VERDE),
        ..Colors::default()
    };
    assert_eq!(
        tintas(&report(), rotulo).map(|inks| inks.label),
        Some(VERDE)
    );
}

#[test]
fn pinta_cada_tom_e_deixa_vazios_e_neutros_sem_escape() {
    let linha = Line::new(vec![
        Segment::new("CPU:", Tone::Label),
        Segment::new(" ", Tone::Plain),
        Segment::new("", Tone::Logo),
        Segment::new("Ryzen", Tone::Value),
        Segment::new("█", Tone::Swatch(VERDE)),
    ]);
    let pintura = Paint::TrueColor(Inks {
        accent: ROSA,
        label: ROSA,
        value: None,
    });
    assert_eq!(
        pintura.line(&linha),
        "\x1b[38;2;255;0;128mCPU:\x1b[0m Ryzen\x1b[38;2;0;255;0m█\x1b[0m"
    );
    assert_eq!(Paint::Plain.line(&linha), "CPU: Ryzen█");
}
