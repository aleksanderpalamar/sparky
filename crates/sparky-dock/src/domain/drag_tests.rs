use super::drag::{Drag, before, nearest};
use super::support::ids;

#[test]
fn so_vira_arrasto_depois_de_mover_alem_da_folga() {
    let mut drag = Drag::new("firefox", 100.0);
    drag.follow(104.0);
    assert!(!drag.moved);
    drag.follow(120.0);
    assert!(drag.moved);
}

#[test]
fn o_destino_e_o_espaco_mais_proximo_do_mouse() {
    let centers = [40.0, 115.0, 190.0];
    assert_eq!(nearest(0.0, &centers), 0);
    assert_eq!(nearest(160.0, &centers), 2);
    assert_eq!(nearest(900.0, &centers), 2);
}

#[test]
fn soltar_em_outro_espaco_diz_antes_de_quem_ele_fica() {
    let order = ids(&["a", "b", "c", "d"]);
    assert_eq!(before(&order, "a", 2), Some(Some("d".to_owned())));
    assert_eq!(before(&order, "d", 0), Some(Some("a".to_owned())));
    assert_eq!(before(&order, "b", 3), Some(None));
    assert_eq!(before(&order, "b", 1), None);
    assert_eq!(before(&order, "x", 1), None);
}
