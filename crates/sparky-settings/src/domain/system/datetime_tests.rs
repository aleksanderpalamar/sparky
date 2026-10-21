use crate::domain::system::datetime::DateTime;
use crate::domain::system::{Change, Control, Request, RowId};
use crate::domain::value::Step;
use crate::ports::fake::FakeCommands;

const SHOW: &str = "Timezone=America/Sao_Paulo\nLocalRTC=no\nCanNTP=yes\nNTP=yes\nNTPSynchronized=yes\nTimeUSec=Wed 2026-10-21 11:35:42 -03\n";

fn sistema() -> FakeCommands {
    FakeCommands::new()
        .with("timedatectl show", SHOW)
        .with("timedatectl list-timezones", "America/Sao_Paulo\nEurope/Lisbon\nUTC\n")
}

#[test]
fn as_linhas_mostram_fuso_hora_e_sincronizacao() {
    let rows = DateTime::load(&sistema()).rows();
    let resumo: Vec<(RowId, &str, &Control)> = rows.iter().map(|row| (row.id, row.value.as_str(), &row.control)).collect();
    assert_eq!(
        resumo,
        [
            (RowId::Timezone, "America/Sao_Paulo", &Control::Text),
            (RowId::LocalTime, "Wed 2026-10-21 11:35:42 -03", &Control::Reading),
            (RowId::NetworkTime, "ligada · sincronizado", &Control::Toggle(true)),
            (RowId::LocalClock, "desligado", &Control::Toggle(false)),
        ]
    );
}

#[test]
fn sem_sincronizacao_possivel_a_linha_so_informa() {
    let clock = DateTime::load(&FakeCommands::new().with("timedatectl show", "CanNTP=no\nNTP=no\n"));
    let ntp = clock.rows().into_iter().find(|row| row.id == RowId::NetworkTime);
    assert_eq!(ntp.map(|row| row.control), Some(Control::Reading));
}

#[test]
fn o_fuso_digitado_precisa_existir() {
    let clock = DateTime::load(&sistema());
    assert_eq!(clock.request(RowId::Timezone, Change::Text(" Europe/Lisbon ".to_owned())), Some(Request::SetTimezone("Europe/Lisbon".to_owned())));
    assert_eq!(clock.request(RowId::Timezone, Change::Text("Marte/Olimpo".to_owned())), None);
    assert_eq!(clock.unknown_timezone(" Marte/Olimpo "), Some("Marte/Olimpo".to_owned()));
    assert_eq!(clock.unknown_timezone("UTC"), None);
}

#[test]
fn os_interruptores_invertem_o_estado() {
    let clock = DateTime::load(&sistema());
    assert_eq!(clock.request(RowId::NetworkTime, Change::Activate), Some(Request::SetNetworkTime(false)));
    assert_eq!(clock.request(RowId::LocalClock, Change::Step(Step::More)), Some(Request::SetLocalClock(true)));
    assert_eq!(clock.request(RowId::LocalTime, Change::Activate), None);
}

#[test]
fn os_pedidos_viram_comandos_do_timedatectl() {
    assert_eq!(Request::SetTimezone("UTC".to_owned()).command(), ("timedatectl", vec!["set-timezone".to_owned(), "UTC".to_owned()]));
    assert_eq!(Request::SetNetworkTime(true).command(), ("timedatectl", vec!["set-ntp".to_owned(), "true".to_owned()]));
    assert_eq!(Request::SetLocalClock(false).command(), ("timedatectl", vec!["set-local-rtc".to_owned(), "false".to_owned()]));
}

#[test]
fn sem_ntp_disponivel_nenhuma_acao_tenta_alterar_a_rede() {
    let clock = DateTime::load(&FakeCommands::new().with("timedatectl show", "CanNTP=no\nNTP=yes\n"));
    for change in [Change::Activate, Change::Step(Step::Less), Change::Step(Step::More)] {
        assert_eq!(clock.request(RowId::NetworkTime, change), None);
    }
}

#[test]
fn descreve_a_rede_desligada_e_a_sincronizacao_em_andamento() {
    for (show, label, enabled) in [
        ("CanNTP=yes\nNTP=no\n", "desligada", false),
        ("CanNTP=yes\nNTP=yes\nNTPSynchronized=no\n", "ligada · sincronizando", true),
    ] {
        let clock = DateTime::load(&FakeCommands::new().with("timedatectl show", show));
        let row = clock.rows().into_iter().find(|row| row.id == RowId::NetworkTime);
        assert_eq!(row.map(|row| (row.value, row.control)), Some((label.to_owned(), Control::Toggle(enabled))));
    }
}

#[test]
fn enter_e_setas_invertem_ambos_os_interruptores() {
    let clock = DateTime::load(&sistema());
    for change in [Change::Activate, Change::Step(Step::Less), Change::Step(Step::More)] {
        assert_eq!(clock.request(RowId::NetworkTime, change.clone()), Some(Request::SetNetworkTime(false)));
        assert_eq!(clock.request(RowId::LocalClock, change), Some(Request::SetLocalClock(true)));
    }
    assert_eq!(clock.request(RowId::NetworkTime, Change::Text("true".to_owned())), None);
    assert_eq!(clock.request(RowId::LocalClock, Change::Text("true".to_owned())), None);
}

#[test]
fn sem_timedatectl_os_dados_ausentes_nao_sao_inventados() {
    let clock = DateTime::load(&FakeCommands::new());
    let rows = clock.rows();
    assert_eq!(rows[0].value, "não definido");
    assert_eq!(rows[1].value, "não definido");
    assert_eq!(rows[2].control, Control::Reading);
    assert_eq!(clock.request(RowId::Timezone, Change::Text("UTC".to_owned())), None);
}

#[test]
fn somente_um_fuso_da_lista_gera_pedido() {
    let clock = DateTime::load(&sistema());
    for input in ["", "  ", "--help", "UTC; reboot", "utc"] {
        assert_eq!(clock.request(RowId::Timezone, Change::Text(input.to_owned())), None);
        assert_eq!(clock.unknown_timezone(input), Some(input.trim().to_owned()));
    }
    assert_eq!(clock.request(RowId::Timezone, Change::Activate), None);
}
