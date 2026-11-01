use crate::screen::{OutputMode, Rotation};

#[test]
fn le_o_modo_com_e_sem_taxa() {
    assert_eq!(
        OutputMode::parse("1920x1080@60.00"),
        Some(OutputMode {
            width: 1920,
            height: 1080,
            refresh: 60_000
        })
    );
    assert_eq!(
        OutputMode::parse(" 2560x1440@143.855 "),
        Some(OutputMode {
            width: 2560,
            height: 1440,
            refresh: 143_855
        })
    );
    assert_eq!(
        OutputMode::parse("1280x720@60Hz"),
        Some(OutputMode {
            width: 1280,
            height: 720,
            refresh: 60_000
        })
    );
    assert_eq!(
        OutputMode::parse("3840x2160"),
        Some(OutputMode {
            width: 3840,
            height: 2160,
            refresh: 0
        })
    );
}

#[test]
fn recusa_modos_invalidos() {
    for texto in [
        "",
        "1920",
        "1920x",
        "x1080",
        "0x1080@60",
        "1920x1080@",
        "1920x1080@abc",
        "1920x1080@-60",
    ] {
        assert_eq!(OutputMode::parse(texto), None, "{texto}");
    }
}

#[test]
fn grava_exato_e_mostra_com_duas_casas() {
    let modo = OutputMode {
        width: 2560,
        height: 1440,
        refresh: 143_855,
    };
    assert_eq!(modo.to_string(), "2560x1440@143.855");
    assert_eq!(modo.label(), "2560x1440@143.86Hz");
    assert_eq!(OutputMode::parse(&modo.to_string()), Some(modo));
}

#[test]
fn reconhece_a_mesma_taxa_com_pequena_diferenca() {
    let gravado = OutputMode {
        width: 1920,
        height: 1080,
        refresh: 60_000,
    };
    assert!(gravado.matches(&OutputMode {
        width: 1920,
        height: 1080,
        refresh: 59_996
    }));
    assert!(!gravado.matches(&OutputMode {
        width: 1920,
        height: 1080,
        refresh: 59_940
    }));
    assert!(!gravado.matches(&OutputMode {
        width: 1280,
        height: 1080,
        refresh: 60_000
    }));
    let sem_taxa = OutputMode {
        width: 1920,
        height: 1080,
        refresh: 0,
    };
    assert!(sem_taxa.matches(&OutputMode {
        width: 1920,
        height: 1080,
        refresh: 144_000
    }));
}

#[test]
fn giros_viram_graus_e_voltam() {
    for (giro, graus) in [
        (Rotation::Normal, 0),
        (Rotation::Right, 90),
        (Rotation::Inverted, 180),
        (Rotation::Left, 270),
    ] {
        assert_eq!(giro.degrees(), graus);
        assert_eq!(Rotation::from_degrees(graus), Some(giro));
    }
    assert_eq!(Rotation::from_degrees(45), None);
    assert!(Rotation::Right.swaps_sides());
    assert!(!Rotation::Inverted.swaps_sides());
}
