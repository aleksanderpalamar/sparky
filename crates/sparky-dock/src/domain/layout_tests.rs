use super::layout::{Geometry, Layout, PEEK, Visibility};

fn layout(slot: i32) -> Layout {
    Layout::new(slot)
}

#[test]
fn com_os_quatro_nativos_no_tamanho_padrao_o_dock_tem_o_tamanho_do_figma() {
    assert_eq!(layout(64).size(4), (408, 100));
    assert_eq!(layout(64).glyph(), 40.0);
}

#[test]
fn cada_aplicativo_novo_alarga_o_dock_em_um_espaco() {
    let dock = layout(64);
    assert_eq!(dock.width(5) - dock.width(4), dock.slot + dock.gap());
}

#[test]
fn o_tamanho_escala_o_icone_e_a_altura() {
    let pequeno = layout(40);
    assert_eq!(pequeno.glyph(), 25.0);
    assert!(pequeno.size(4).1 < layout(64).size(4).1);
    assert!(layout(96).size(4).0 > layout(64).size(4).0);
}

#[test]
fn visivel_e_sempre_presente_reserva_a_altura_e_flutua_acima_da_borda() {
    let geometry = layout(64).geometry(4, Visibility::Shown, false);
    assert_eq!(
        geometry,
        Geometry {
            size: (408, 117),
            zone: 117
        }
    );
}

#[test]
fn ocultando_automaticamente_nao_reserva_espaco_e_escondido_vira_uma_faixa() {
    let shown = layout(64).geometry(4, Visibility::Shown, true);
    assert_eq!(shown.zone, 0);
    let hidden = layout(64).geometry(4, Visibility::Hidden, true);
    assert_eq!(
        hidden,
        Geometry {
            size: (408, PEEK),
            zone: 0
        }
    );
}

#[test]
fn com_o_menu_aberto_a_superficie_cresce_para_cima_e_cabe_o_menu() {
    let dock = layout(40);
    let base = dock.geometry(1, Visibility::Shown, false);
    let open = base.with_menu();
    assert_eq!(open.size.1, base.size.1 + super::layout::MENU_SPACE);
    assert!(open.size.0 >= super::layout::MENU_WIDTH as u32);
    assert_eq!(open.zone, base.zone);
}

#[test]
fn revelado_o_dock_continua_tocando_a_borda_onde_o_mouse_esta() {
    let shown = layout(64).geometry(4, Visibility::Shown, true);
    assert_eq!(
        shown.size.1,
        layout(64).size(4).1 + super::layout::LIFT as u32
    );
    assert!(shown.size.1 > PEEK);
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
fn com_o_mouse_no_dock_a_superficie_ganha_espaco_para_os_icones_ampliados() {
    let dock = layout(64);
    let base = dock.geometry(4, Visibility::Shown, false);
    let lifted = base.lifted(&dock);
    assert!(lifted.size.1 as f32 >= base.size.1 as f32 + 64.0 * 0.6);
    assert!(lifted.size.0 > base.size.0);
    assert_eq!(lifted.zone, base.zone);
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
