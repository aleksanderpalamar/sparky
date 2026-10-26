use super::{Encoding, MAX_STEPS, wheel};
use crate::domain::grid::{Half, Spot};
use crate::domain::input::Wheel;

fn spot(column: usize, line: usize) -> Spot {
    Spot {
        column,
        line,
        half: Half::Left,
    }
}

#[test]
fn sgr_envia_direcao_e_coordenadas_da_celula_com_base_um() {
    assert_eq!(
        wheel(1.0, 0.0, spot(4, 2), Encoding::Sgr).0,
        Wheel::Keys(b"\x1b[<64;5;3M".to_vec())
    );
    assert_eq!(
        wheel(-2.0, 0.0, spot(250, 300), Encoding::Sgr).0,
        Wheel::Keys(b"\x1b[<65;251;301M\x1b[<65;251;301M".to_vec())
    );
}

#[test]
fn fracao_da_roda_so_envia_evento_ao_completar_um_passo() {
    let (first, rest) = wheel(0.5, 0.0, spot(0, 0), Encoding::Sgr);
    assert_eq!(first, Wheel::Keys(Vec::new()));
    assert_eq!(rest, 0.5);
    assert_eq!(
        wheel(0.75, rest, spot(0, 0), Encoding::Sgr),
        (Wheel::Keys(b"\x1b[<64;1;1M".to_vec()), 0.25)
    );
    assert_eq!(
        wheel(-1.5, 0.25, spot(0, 0), Encoding::Sgr),
        (Wheel::Keys(b"\x1b[<65;1;1M".to_vec()), -0.25)
    );
}

#[test]
fn protocolo_legado_envia_bytes_crus_e_respeita_o_limite_de_coordenadas() {
    assert_eq!(
        wheel(1.0, 0.0, spot(0, 0), Encoding::Legacy).0,
        Wheel::Keys(b"\x1b[M`!!".to_vec())
    );
    assert_eq!(
        wheel(-1.0, 0.0, spot(222, 222), Encoding::Legacy).0,
        Wheel::Keys(vec![27, 91, 77, 97, 255, 255])
    );
    assert_eq!(
        wheel(1.0, 0.0, spot(223, 0), Encoding::Legacy).0,
        Wheel::Keys(Vec::new())
    );
}

#[test]
fn protocolo_utf8_codifica_as_coordenadas_estendidas() {
    assert_eq!(
        wheel(-1.0, 0.0, spot(94, 2014), Encoding::Utf8).0,
        Wheel::Keys("\x1b[Ma\u{7f}\u{7ff}".as_bytes().to_vec())
    );
    assert_eq!(
        wheel(1.0, 0.0, spot(2015, 0), Encoding::Utf8).0,
        Wheel::Keys(Vec::new())
    );
}

#[test]
fn coordenadas_inrepresentaveis_nao_geram_relatorio_parcial() {
    for encoding in [Encoding::Sgr, Encoding::Utf8, Encoding::Legacy] {
        for at in [spot(usize::MAX, 0), spot(0, usize::MAX)] {
            assert_eq!(wheel(1.0, 0.0, at, encoding).0, Wheel::Keys(Vec::new()));
        }
    }
}

#[test]
fn um_evento_nao_pode_alocar_relatorios_sem_limite() {
    assert_eq!(
        wheel(1_000_000.0, 0.0, spot(0, 0), Encoding::Sgr).0,
        Wheel::Keys(b"\x1b[<64;1;1M".repeat(MAX_STEPS))
    );
}
