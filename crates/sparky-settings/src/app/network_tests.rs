use std::rc::Rc;

use sparky_ui::notice::Notice;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::app::test_support::{Comandos, Sparky, painel_com};
use crate::app::update::update;
use crate::domain::network::{AutoDns, Draft, Field, Method};
use crate::domain::section::Section;
use crate::ports::fake::FakeCommands;

pub(super) const ATIVAS: &str = "nmcli -t -f UUID,DEVICE connection show --active";
pub(super) const DETALHES: &str = "nmcli -e no -g connection.id,ipv4.method,ipv4.addresses,ipv4.gateway,ipv4.dns,ipv4.ignore-auto-dns connection show";
const REAPLICAR: &str = "nmcli device reapply enp7s0";
pub(super) const CONCESSAO: &str = "nmcli -g IP4.ADDRESS,IP4.GATEWAY,IP4.DNS device show enp7s0";

fn gravacao() -> String {
    let draft = Draft {
        method: Method::Manual,
        address: "10.0.0.2/24".to_owned(),
        gateway: "10.0.0.1".to_owned(),
        primary_dns: "1.1.1.1".to_owned(),
        secondary_dns: String::new(),
        auto_dns: AutoDns::Use,
    };
    format!(
        "nmcli {}",
        draft.arguments("uuid-2").unwrap_or_default().join(" ")
    )
}

fn conexoes() -> FakeCommands {
    FakeCommands::new()
        .with(ATIVAS, "uuid-1:wlan0\nuuid-2:enp7s0\n")
        .with(
            &format!("{DETALHES} uuid-1"),
            "Wi-Fi\nmanual\n192.168.1.5/24\n\n\n",
        )
        .with(
            &format!("{DETALHES} uuid-2"),
            "Cabo\nmanual\n10.0.0.2/24\n10.0.0.1\n\n",
        )
        .accepting(&gravacao())
}

fn salvar_dns_da_segunda(commands: &Comandos) -> Settings {
    let (mut settings, _) = painel_com(None, Sparky::default(), commands.clone());
    let _ = update(&mut settings, Message::Section(Section::Network));
    let _ = update(&mut settings, Message::NetworkSelect(1));
    let _ = update(
        &mut settings,
        Message::NetworkInput(Field::PrimaryDns, "1.1.1.1".to_owned()),
    );
    let _ = update(&mut settings, Message::NetworkSave);
    settings
}

#[test]
fn salvar_reaplica_a_interface_e_mantem_a_conexao_escolhida() {
    let commands = Comandos(Rc::new(conexoes().accepting(REAPLICAR)));
    let settings = salvar_dns_da_segunda(&commands);
    let calls = commands.0.calls();
    assert!(calls.contains(&gravacao()));
    assert!(calls.contains(&REAPLICAR.to_owned()));
    assert_eq!(settings.network.selected, 1);
    assert_eq!(
        settings.notice,
        Some(Notice::Info("Configuração IPv4 aplicada".to_owned()))
    );
}

#[test]
fn sem_reaplicacao_avisa_que_e_preciso_reconectar() {
    let commands = Comandos(Rc::new(conexoes()));
    let settings = salvar_dns_da_segunda(&commands);
    assert!(commands.0.calls().contains(&REAPLICAR.to_owned()));
    assert_eq!(
        settings.notice,
        Some(Notice::Info(
            "Configuração IPv4 salva; reconecte a rede para aplicar".to_owned()
        ))
    );
}

#[test]
fn endereco_invalido_nao_grava_nem_reaplica() {
    let commands = Comandos(Rc::new(conexoes().accepting(REAPLICAR)));
    let (mut settings, _) = painel_com(None, Sparky::default(), commands.clone());
    let _ = update(&mut settings, Message::Section(Section::Network));
    let _ = update(
        &mut settings,
        Message::NetworkInput(Field::Address, "10.0.0.2".to_owned()),
    );
    let _ = update(&mut settings, Message::NetworkSave);
    assert!(matches!(&settings.notice, Some(Notice::Error(text)) if text.contains("prefixo")));
    assert!(
        commands
            .0
            .calls()
            .iter()
            .all(|call| !call.contains("modify") && !call.contains("reapply"))
    );
}

#[test]
fn dhcp_pode_usar_somente_os_dns_informados() {
    let gravacao = Draft {
        method: Method::Auto,
        address: String::new(),
        gateway: String::new(),
        primary_dns: "9.9.9.9".to_owned(),
        secondary_dns: String::new(),
        auto_dns: AutoDns::Ignore,
    }
    .arguments("uuid-3")
    .unwrap_or_default()
    .join(" ");
    let commands = Comandos(Rc::new(
        FakeCommands::new()
            .with(ATIVAS, "uuid-3:wlan0\n")
            .with(&format!("{DETALHES} uuid-3"), "Casa\nauto\n\n\n\nno\n")
            .accepting(&format!("nmcli {gravacao}")),
    ));
    let (mut settings, _) = painel_com(None, Sparky::default(), commands.clone());
    let _ = update(&mut settings, Message::Section(Section::Network));
    let _ = update(&mut settings, Message::NetworkAutoDns(AutoDns::Ignore));
    let _ = update(
        &mut settings,
        Message::NetworkInput(Field::PrimaryDns, "9.9.9.9".to_owned()),
    );
    let _ = update(&mut settings, Message::NetworkSave);
    assert!(commands.0.calls().contains(&format!("nmcli {gravacao}")));
    assert!(gravacao.ends_with("ipv4.ignore-auto-dns yes"));
}
