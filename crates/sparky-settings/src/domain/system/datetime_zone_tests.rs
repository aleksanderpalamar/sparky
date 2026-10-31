use super::datetime_tests::{SHOW, sistema};
use crate::domain::system::datetime::DateTime;
use crate::domain::system::{Change, Control, Request, RowId};
use crate::ports::fake::FakeCommands;

#[test]
fn o_fuso_digitado_precisa_existir() {
    let clock = DateTime::load(&sistema());
    assert_eq!(
        clock.request(RowId::Timezone, Change::Text(" Europe/Lisbon ".to_owned())),
        Some(Request::SetTimezone("Europe/Lisbon".to_owned()))
    );
    assert_eq!(
        clock.request(RowId::Timezone, Change::Text("Marte/Olimpo".to_owned())),
        None
    );
    assert_eq!(
        clock.unknown_timezone(" Marte/Olimpo "),
        Some("Marte/Olimpo".to_owned())
    );
    assert_eq!(clock.unknown_timezone("UTC"), None);
}

#[test]
fn somente_um_fuso_da_lista_gera_pedido() {
    let clock = DateTime::load(&sistema());
    for input in ["", "  ", "--help", "UTC; reboot", "utc"] {
        assert_eq!(
            clock.request(RowId::Timezone, Change::Text(input.to_owned())),
            None
        );
        assert_eq!(clock.unknown_timezone(input), Some(input.trim().to_owned()));
    }
    assert_eq!(clock.request(RowId::Timezone, Change::Activate), None);
}

#[test]
fn sem_lista_de_fusos_o_fuso_e_digitado() {
    let commands = FakeCommands::new().with("timedatectl show", SHOW);
    assert_eq!(DateTime::load(&commands).rows()[0].control, Control::Text);
}
