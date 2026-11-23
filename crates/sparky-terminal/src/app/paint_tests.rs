use sparky_theme::Rgb;

use super::paint::backdrop;

#[test]
fn o_fundo_do_terminal_segue_a_opacidade_do_tema() {
    let fundo = Rgb::new(10, 20, 30);
    assert_eq!(backdrop(fundo, 1.0).a, 1.0);
    assert_eq!(backdrop(fundo, 0.72).a, 0.72);
    assert_eq!(backdrop(fundo, 3.0).a, 1.0);
    assert_eq!(backdrop(fundo, 0.72).r, backdrop(fundo, 1.0).r);
}
