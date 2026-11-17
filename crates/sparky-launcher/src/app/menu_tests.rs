use crate::app::navigation::Scroll;
use crate::app::test_support::{Memory, com_grupos, grupo, launcher};
use crate::app::view::{MENU_ITEM, menu_height};
use crate::domain::overlay::Overlay;

#[test]
fn o_cursor_do_menu_para_nas_pontas() {
    let mut launcher = launcher(&["Steam"], &Memory::default());
    launcher.open_menu_on_chosen();
    assert_eq!(launcher.overlay, Overlay::Closed);
    launcher.hover(0);
    launcher.open_menu_on_chosen();
    let ultimo = launcher.menu_items().len() - 1;
    launcher.menu_step(50);
    assert!(matches!(&launcher.overlay, Overlay::Menu(menu) if menu.cursor == ultimo));
    launcher.menu_step(-90);
    assert!(matches!(&launcher.overlay, Overlay::Menu(menu) if menu.cursor == 0));
}

#[test]
fn com_muitos_grupos_o_menu_rola_ate_o_item_marcado() {
    let lista = (0..20)
        .map(|index| grupo(&format!("Grupo {index}"), &[]))
        .collect();
    let memoria = com_grupos(lista);
    let mut launcher = launcher(&["Emojify"], &memoria);
    launcher.open_app_menu("emojify");
    let total = launcher.menu_items().len();
    assert_eq!(total, 26);
    let rolagens: Vec<Scroll> = (1..total).map(|_| launcher.menu_step(1)).collect();
    assert_eq!(rolagens[0], Scroll::Keep);
    let Some(Scroll::To(fim)) = rolagens.last().copied() else {
        panic!("o último item deveria pedir rolagem");
    };
    let altura = menu_height(total, launcher.mode);
    assert!(altura < total as f32 * MENU_ITEM);
    assert_eq!(fim, total as f32 * MENU_ITEM - altura);
    launcher.menu_scrolled(fim);
    assert_eq!(launcher.menu_step(-30), Scroll::To(0.0));
}
