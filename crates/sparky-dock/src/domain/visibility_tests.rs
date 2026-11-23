use super::layout::Visibility;
use super::presence::Seen;
use super::visibility::{Moment, visibility};

fn window(visible: bool) -> Seen {
    Seen {
        app_id: "firefox".to_owned(),
        focused: false,
        visible,
    }
}

fn moment(autohide: bool, windows: &[Seen], hovering: bool, busy: bool) -> Visibility {
    visibility(&Moment {
        autohide,
        windows,
        hovering,
        busy,
    })
}

#[test]
fn sem_ocultar_automaticamente_o_dock_fica_sempre_visivel() {
    assert_eq!(
        moment(false, &[window(true)], false, false),
        Visibility::Shown
    );
}

#[test]
fn com_janela_visivel_o_dock_se_esconde() {
    assert_eq!(
        moment(true, &[window(true)], false, false),
        Visibility::Hidden
    );
}

#[test]
fn sem_janela_visivel_o_dock_aparece() {
    assert_eq!(moment(true, &[], false, false), Visibility::Shown);
    assert_eq!(
        moment(true, &[window(false)], false, false),
        Visibility::Shown
    );
}

#[test]
fn o_mouse_em_cima_ou_um_menu_aberto_seguram_o_dock() {
    assert_eq!(
        moment(true, &[window(true)], true, false),
        Visibility::Shown
    );
    assert_eq!(
        moment(true, &[window(true)], false, true),
        Visibility::Shown
    );
}
