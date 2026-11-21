use super::layout::{GAP, SLOT, height, reserved, size, width};

#[test]
fn cada_aplicativo_novo_alarga_o_dock_em_um_espaco() {
    assert_eq!(width(5) - width(4), SLOT + GAP);
    assert_eq!(size(0).0, width(0).ceil() as u32);
}

#[test]
fn a_altura_nao_depende_da_quantidade_e_a_reserva_e_so_o_dock() {
    assert_eq!(size(1).1, size(9).1);
    assert_eq!(reserved(), height().ceil() as i32);
}
