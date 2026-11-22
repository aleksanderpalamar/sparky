use super::layout::{GAP, Geometry, Layout, MENU_SPACE};

fn layout(slot: i32) -> Layout {
    Layout::new(slot)
}

#[test]
fn com_os_quatro_nativos_no_tamanho_padrao_o_dock_tem_o_tamanho_do_figma() {
    assert_eq!(layout(64).size(4), (408, 100));
    assert_eq!(layout(64).glyph(), 40.0);
    assert_eq!(layout(64).rest(), 117);
}

#[test]
fn cada_aplicativo_novo_alarga_o_dock_em_um_espaco() {
    let dock = layout(64);
    assert_eq!(dock.width(5) - dock.width(4), dock.slot + GAP);
}

#[test]
fn o_tamanho_escala_o_icone_e_a_altura() {
    let pequeno = layout(40);
    assert_eq!(pequeno.glyph(), 25.0);
    assert!(pequeno.size(4).1 < layout(64).size(4).1);
    assert!(layout(96).size(4).0 > layout(64).size(4).0);
}

#[test]
fn o_quadro_e_fixo_e_cabe_os_icones_ampliados_e_o_menu() {
    let dock = layout(64);
    let frame = dock.frame(4, false);
    assert_eq!(frame.size.1, 117 + dock.rise() + MENU_SPACE);
    assert!(frame.size.0 >= 408 + dock.rise() * 3);
    assert!(dock.rise() as f32 >= 64.0 * 0.6);
}

#[test]
fn so_o_dock_sempre_visivel_reserva_espaco_na_tela() {
    let dock = layout(64);
    assert_eq!(dock.frame(4, false).zone, 117);
    let oculto = dock.frame(4, true);
    assert_eq!(
        oculto,
        Geometry {
            zone: 0,
            ..dock.frame(4, false)
        }
    );
}

#[test]
fn com_poucos_apps_o_quadro_ainda_cabe_o_menu() {
    assert!(layout(40).frame(0, false).size.0 >= 200);
}

#[test]
fn os_centros_seguem_a_grade_e_o_mais_fica_depois_do_divisor() {
    let dock = layout(64);
    let centers = dock.centers(2, 300.0);
    let left = (300.0 - dock.width(2)) / 2.0;
    assert_eq!(centers.len(), 3);
    assert_eq!(centers[0], left + 16.0 + 32.0);
    assert_eq!(centers[1] - centers[0], 75.0);
    assert_eq!(centers[2] - centers[1], 75.0 + 1.0 + 11.0);
}

#[test]
fn o_fundo_ampliado_soma_a_largura_de_cada_icone() {
    let dock = layout(64);
    assert_eq!(dock.tray_width(&[1.0; 5]), dock.width(4));
    assert_eq!(
        dock.tray_width(&[1.5, 1.0, 1.0, 1.0, 1.0]),
        dock.width(4) + 32.0
    );
}
