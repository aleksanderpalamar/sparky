use std::rc::Rc;

use crate::app::message::Message;
use crate::app::network_tests::{ATIVAS, CONCESSAO, DETALHES};
use crate::app::state::Settings;
use crate::app::test_support::{Comandos, Sparky, painel_com};
use crate::app::text_probe::visible_texts;
use crate::app::update::update;
use crate::domain::network::Method;
use crate::domain::section::Section;
use crate::ports::fake::FakeCommands;

fn cabo_com_dhcp(metodo: Method) -> Settings {
    let commands = Comandos(Rc::new(
        FakeCommands::new()
            .with(ATIVAS, "uuid-4:enp7s0\n")
            .with(&format!("{DETALHES} uuid-4"), "Cabo\nauto\n\n\n\nno\n")
            .with(CONCESSAO, "192.168.0.70/24\n192.168.0.1\n192.168.0.1\n"),
    ));
    let (mut settings, _) = painel_com(None, Sparky::default(), commands);
    let _ = update(&mut settings, Message::Section(Section::Network));
    let _ = update(&mut settings, Message::NetworkMethod(metodo));
    settings
}

fn confere(textos: &[String], presentes: &[&str], ausentes: &[&str]) {
    for texto in presentes {
        assert!(textos.iter().any(|t| t == texto), "faltou {texto}");
    }
    for texto in ausentes {
        assert!(textos.iter().all(|t| t != texto), "sobrou {texto}");
    }
}

#[test]
fn escolher_manual_na_tela_parte_do_ip_recebido_do_roteador() {
    let settings = cabo_com_dhcp(Method::Manual);
    let Some(draft) = &settings.network.draft else {
        panic!("rascunho IPv4 ausente");
    };
    assert_eq!(draft.method, Method::Manual);
    assert_eq!(draft.address, "192.168.0.70/24");
    assert_eq!(draft.gateway, "192.168.0.1");
    assert_eq!(draft.primary_dns, "192.168.0.1");
}

#[test]
fn dhcp_mostra_ip_gateway_e_dns_recebidos_do_roteador() {
    let settings = cabo_com_dhcp(Method::Auto);
    let textos = visible_texts(crate::view::network::view(&settings));
    confere(
        &textos,
        &[
            "Recebido do roteador",
            "192.168.0.70/24",
            "192.168.0.1",
            "DNS do DHCP",
            "DNS primário",
            "DNS secundário",
        ],
        &["Endereço IPv4 / prefixo"],
    );
}

#[test]
fn manual_mostra_campos_editaveis_e_dois_dns() {
    let settings = cabo_com_dhcp(Method::Manual);
    let textos = visible_texts(crate::view::network::view(&settings));
    confere(
        &textos,
        &[
            "Endereço IPv4 / prefixo",
            "Gateway",
            "DNS primário",
            "DNS secundário",
        ],
        &["Recebido do roteador", "DNS do DHCP"],
    );
}
