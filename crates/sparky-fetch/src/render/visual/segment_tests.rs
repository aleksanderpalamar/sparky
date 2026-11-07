use crate::render::visual::segment::{Line, Segment, Tone};

#[test]
fn mede_e_junta_segmentos() {
    let linha = Line::new(vec![Segment::new("CPU:", Tone::Label)])
        .then(Line::new(vec![Segment::new(" Ryzen", Tone::Value)]));
    assert_eq!(linha.width(), 10);
    assert_eq!(linha.segments.len(), 2);
}

#[test]
fn remove_espacos_e_segmentos_vazios_do_fim() {
    let linha = Line::new(vec![
        Segment::new("Disco:", Tone::Label),
        Segment::new("  ", Tone::Plain),
        Segment::new("", Tone::Value),
    ]);
    assert_eq!(
        linha.trimmed(),
        Line::new(vec![Segment::new("Disco:", Tone::Label)])
    );
    assert_eq!(Line::blank().trimmed(), Line::default());
}
