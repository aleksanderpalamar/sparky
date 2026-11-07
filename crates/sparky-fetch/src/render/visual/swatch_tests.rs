use sparky_theme::Rgb;

use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::{Status, Value};
use crate::render::visual::fixture::{ACCENT, palette, report};
use crate::render::visual::segment::{Line, Segment, Tone};
use crate::render::visual::style::{Inks, Paint};
use crate::render::visual::swatch;

const VERMELHO: Rgb = Rgb::new(0xf7, 0x76, 0x8e);
const VERDE: Rgb = Rgb::new(0x9e, 0xce, 0x6a);

fn colorido() -> Paint {
    Paint::TrueColor(Inks {
        accent: ACCENT,
        label: ACCENT,
        value: None,
    })
}

#[test]
fn a_paleta_vem_do_campo_de_cores_pronto() {
    assert_eq!(swatch::palette(&report()), Some(palette().as_slice()));
    let texto = Report {
        entries: vec![Entry::new(Field::Colors, Status::text("#ffffff"))],
    };
    assert_eq!(swatch::palette(&texto), None);
    assert!(swatch::is_palette(&report().entries[10]));
    assert!(!swatch::is_palette(&texto.entries[0]));
}

#[test]
fn as_amostras_pulam_o_destaque() {
    let so_destaque = Report {
        entries: vec![Entry::new(
            Field::Colors,
            Status::ready(Value::Colors(vec![ACCENT])),
        )],
    };
    assert_eq!(swatch::samples(&so_destaque), None);
    assert_eq!(swatch::samples(&report()).map(<[Rgb]>::len), Some(4));
}

#[test]
fn sem_cor_mostra_os_codigos() {
    let linha = swatch::row(&[VERMELHO, VERDE], &Paint::Plain, 80);
    assert_eq!(
        linha,
        Line::new(vec![
            Segment::new("#f7768e", Tone::Swatch(VERMELHO)),
            Segment::new(" ", Tone::Plain),
            Segment::new("#9ece6a", Tone::Swatch(VERDE)),
        ])
    );
}

#[test]
fn com_cor_mostra_blocos_e_so_os_que_cabem() {
    let linha = swatch::row(&[VERMELHO, VERDE], &colorido(), 80);
    assert_eq!(
        linha,
        Line::new(vec![
            Segment::new("███", Tone::Swatch(VERMELHO)),
            Segment::new(" ", Tone::Plain),
            Segment::new("███", Tone::Swatch(VERDE)),
        ])
    );
    let estreita = swatch::row(&[VERMELHO, VERDE], &colorido(), 6);
    assert_eq!(
        estreita,
        Line::new(vec![Segment::new("███", Tone::Swatch(VERMELHO))])
    );
}
