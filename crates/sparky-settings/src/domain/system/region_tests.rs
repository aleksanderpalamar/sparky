use crate::domain::system::region::Region;
use crate::domain::system::{Change, Control, Request, RowId};
use crate::domain::value::Step;
use crate::ports::fake::FakeCommands;

pub fn sistema() -> FakeCommands {
    FakeCommands::new()
        .with("localectl status", "System Locale: LC_TIME=en_US.UTF-8 LANG=pt_BR.UTF-8\nVC Keymap: br-abnt2\nX11 Layout: br\n")
        .with("localectl list-locales", "C.UTF-8\npt_BR.UTF-8\n")
        .with("localectl list-keymaps", "br-abnt2\nus\n")
        .with("localectl list-x11-keymap-layouts", "br\nus\n")
}

#[test]
fn mostra_idioma_e_teclados_com_as_opcoes_disponiveis() {
    let rows = Region::load(&sistema()).rows();
    let values: Vec<_> = rows.iter().map(|row| (row.id, row.value.as_str(), &row.control)).collect();
    assert_eq!(values, [
        (RowId::Locale, "pt_BR.UTF-8", &Control::Text),
        (RowId::ConsoleKeymap, "br-abnt2", &Control::Text),
        (RowId::SystemLayout, "br", &Control::Text),
    ]);
}

#[test]
fn somente_valores_listados_podem_ser_digitados() {
    let region = Region::load(&sistema());
    assert_eq!(region.request(RowId::Locale, Change::Text(" C.UTF-8 ".to_owned())), Some(Request::SetLocale("C.UTF-8".to_owned())));
    assert_eq!(region.request(RowId::ConsoleKeymap, Change::Text("us".to_owned())), Some(Request::SetConsoleKeymap("us".to_owned())));
    assert_eq!(region.request(RowId::SystemLayout, Change::Text("us".to_owned())), Some(Request::SetSystemLayout("us".to_owned())));
    for input in ["", "fr", "--help", "br; reboot"] {
        assert_eq!(region.request(RowId::SystemLayout, Change::Text(input.to_owned())), None);
        assert_eq!(region.unknown_value(RowId::SystemLayout, input), Some(input.to_owned()));
    }
}

#[test]
fn setas_percorrem_as_opcoes_e_dao_a_volta() {
    let region = Region::load(&sistema());
    assert_eq!(region.request(RowId::Locale, Change::Step(Step::More)), Some(Request::SetLocale("C.UTF-8".to_owned())));
    assert_eq!(region.request(RowId::ConsoleKeymap, Change::Step(Step::Less)), Some(Request::SetConsoleKeymap("us".to_owned())));
    assert_eq!(region.request(RowId::Timezone, Change::Step(Step::More)), None);
}

#[test]
fn sem_opcoes_a_linha_e_somente_leitura() {
    let region = Region::load(&FakeCommands::new());
    assert!(region.rows().iter().all(|row| row.value == "não definido" && row.control == Control::Reading));
    assert_eq!(region.request(RowId::Locale, Change::Step(Step::More)), None);
}

#[test]
fn os_pedidos_usam_argumentos_separados_do_localectl() {
    assert_eq!(Request::SetLocale("C.UTF-8".to_owned()).command(), ("localectl", vec!["set-locale".to_owned(), "LANG=C.UTF-8".to_owned()]));
    assert_eq!(Request::SetConsoleKeymap("us".to_owned()).command(), ("localectl", vec!["set-keymap".to_owned(), "us".to_owned()]));
    assert_eq!(Request::SetSystemLayout("br".to_owned()).command(), ("localectl", vec!["set-x11-keymap".to_owned(), "br".to_owned()]));
}
