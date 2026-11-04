use crate::domain::inhibitions::{Hold, Inhibitions};

#[test]
fn o_primeiro_pedido_segura_e_o_ultimo_solta() {
    let mut pedidos = Inhibitions::default();
    let (primeiro, mudou) = pedidos.inhibit(":1.10");
    assert_eq!(mudou, Some(Hold::Held));
    let (segundo, mudou) = pedidos.inhibit(":1.11");
    assert_eq!(mudou, None);
    assert_ne!(primeiro, segundo);
    assert_eq!(pedidos.release(primeiro), None);
    assert_eq!(pedidos.release(segundo), Some(Hold::Free));
    assert_eq!(pedidos.hold(), Hold::Free);
}

#[test]
fn soltar_um_pedido_desconhecido_nao_muda_nada() {
    let mut pedidos = Inhibitions::default();
    assert_eq!(pedidos.release(42), None);
    let (_, _) = pedidos.inhibit(":1.10");
    assert_eq!(pedidos.release(42), None);
    assert_eq!(pedidos.hold(), Hold::Held);
}

#[test]
fn quem_sai_do_barramento_leva_os_seus_pedidos() {
    let mut pedidos = Inhibitions::default();
    let _ = pedidos.inhibit(":1.10");
    let _ = pedidos.inhibit(":1.10");
    let (outro, _) = pedidos.inhibit(":1.20");
    assert_eq!(pedidos.forget(":1.10"), None);
    assert_eq!(pedidos.forget(":1.99"), None);
    assert_eq!(pedidos.release(outro), Some(Hold::Free));
}

#[test]
fn o_ultimo_dono_saindo_solta() {
    let mut pedidos = Inhibitions::default();
    let _ = pedidos.inhibit(":1.10");
    assert_eq!(pedidos.forget(":1.10"), Some(Hold::Free));
}

#[test]
fn o_numero_do_pedido_nunca_e_zero() {
    let mut pedidos = Inhibitions::default();
    let (cookie, _) = pedidos.inhibit(":1.10");
    assert_ne!(cookie, 0);
}
