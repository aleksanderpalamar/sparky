use std::io;
use std::rc::Rc;

use sparky_theme::{Palette, Skin};
use sparky_ui::notice::Notice;

use crate::app::message::Message;
use crate::app::state::{Mode, Settings};
use crate::app::test_support::{Comandos, Sparky, enter, painel_com};
use crate::app::update::update;
use crate::domain::section::Section;
use crate::domain::system::themes::Theme;
use crate::ports::fake::FakeCommands;
use crate::ports::fake_themes::FakeThemes;

const ICONES: &str = "gsettings set org.gnome.desktop.interface icon-theme Yaru-blue";

fn painel() -> (Settings, Rc<FakeThemes>, Comandos) {
    painel_com_tema(None, Comandos::default())
}

fn painel_com_tema(icones: Option<&str>, commands: Comandos) -> (Settings, Rc<FakeThemes>, Comandos) {
    let (mut settings, _) = painel_com(None, Sparky::default(), commands.clone());
    let store = Rc::new(FakeThemes::default());
    let theme = Theme::parse("azul", "accent = \"#010203\"").unwrap_or_else(|_| Theme::builtin());
    store.catalog.borrow_mut().themes.push(theme.with_icons(icones.map(str::to_owned)));
    settings.services.themes = Box::new(store.clone());
    settings.reload_system();
    settings.go(Section::Themes);
    settings.point(1);
    (settings, store, commands)
}

#[test]
fn aplicar_paleta_atualiza_o_painel_sem_comandos_externos() {
    let (mut settings, store, commands) = painel();
    let before = commands.0.calls();
    enter(&mut settings);
    assert_eq!(*store.applied.borrow(), ["azul"]);
    assert_eq!(settings.context.skin, Skin::new(store.catalog.borrow().current));
    assert_eq!(commands.0.calls(), before);
    assert_eq!(settings.notice, Some(Notice::Info("Paleta azul aplicada.".to_owned())));
    assert_eq!(settings.mode, Mode::Browse);
    enter(&mut settings);
    assert_eq!(store.applied.borrow().len(), 1);
}

#[test]
fn falha_de_gravacao_preserva_as_cores_anteriores() {
    let (mut settings, store, _) = painel();
    let original = settings.context.skin;
    store.failure.set(Some(io::ErrorKind::PermissionDenied));
    enter(&mut settings);
    assert_eq!(settings.context.skin, original);
    assert!(store.applied.borrow().is_empty());
    assert!(matches!(&settings.notice, Some(Notice::Error(text)) if text.contains("Paleta azul:")));
}

#[test]
fn evento_de_paleta_preserva_edicao_e_foco() {
    let (mut settings, _, _) = painel();
    settings.go(Section::Apps);
    enter(&mut settings);
    settings.input = "kitty".to_owned();
    let mode = settings.mode;
    let palette = Palette::from_colors("accent = \"#123456\"");
    let _ = update(&mut settings, Message::Palette(palette));
    assert_eq!(settings.context.skin, Skin::new(palette));
    assert_eq!(settings.input, "kitty");
    assert_eq!(settings.mode, mode);
    assert_eq!(settings.focus, 0);
}

#[test]
fn evento_de_paleta_tambem_atualiza_a_marca_em_uso() {
    let (mut settings, store, _) = painel();
    let palette = store.catalog.borrow().themes[1].palette();
    let _ = update(&mut settings, Message::Palette(palette));
    let rows = settings.themes.rows();
    assert_eq!(rows[0].control, crate::domain::system::Control::Toggle(false));
    assert_eq!(rows[1].control, crate::domain::system::Control::Toggle(true));
}

#[test]
fn aplicar_um_tema_com_icones_troca_os_icones_pelo_gsettings() {
    let commands = Comandos(Rc::new(FakeCommands::new().accepting(ICONES)));
    let (mut settings, store, commands) = painel_com_tema(Some("Yaru-blue"), commands);
    enter(&mut settings);
    assert_eq!(*store.applied.borrow(), ["azul"]);
    assert!(commands.0.calls().iter().any(|call| call == ICONES));
    assert_eq!(settings.notice, Some(Notice::Info("Paleta azul aplicada com os ícones Yaru-blue.".to_owned())));
}

#[test]
fn sem_gsettings_a_paleta_fica_e_o_aviso_explica_os_icones() {
    let (mut settings, store, _) = painel_com_tema(Some("Yaru-blue"), Comandos::default());
    enter(&mut settings);
    assert_eq!(*store.applied.borrow(), ["azul"]);
    assert_eq!(settings.context.skin, Skin::new(store.catalog.borrow().current));
    assert!(matches!(&settings.notice, Some(Notice::Error(text)) if text.starts_with("Paleta azul aplicada, mas os ícones Yaru-blue não:")));
}
