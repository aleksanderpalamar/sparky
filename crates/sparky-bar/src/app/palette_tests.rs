use sparky_theme::{Palette, Skin};

use crate::app::message::Message;
use crate::app::tray_test_support::{bar, messages, opened};
use crate::app::update::update;

#[test]
fn a_paleta_muda_sem_fechar_o_menu_da_bandeja() {
    let mut bar = bar();
    let _ = opened(&mut bar, ":1.1");
    let palette = Palette::from_colors("accent = \"#010203\"");
    let output = messages(update(&mut bar, Message::Palette(palette)));
    assert_eq!(*bar.skin(), Skin::new(palette));
    assert!(bar.tray().menu().is_some());
    assert!(output.is_empty());
}
