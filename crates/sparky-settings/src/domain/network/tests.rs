use super::{Draft, Field, Method};
use crate::ports::fake::FakeCommands;

const ATIVAS: &str = "nmcli -t -f UUID,DEVICE connection show --active";

fn detalhes(uuid: &str) -> String {
    format!(
        "nmcli -e no -g connection.id,ipv4.method,ipv4.addresses,ipv4.gateway,ipv4.dns connection show {uuid}"
    )
}

#[test]
fn rejeita_endereco_prefixo_gateway_e_dns_invalidos() {
    let mut draft = Draft {
        method: Method::Manual,
        address: "inválido".to_owned(),
        gateway: String::new(),
        dns: String::new(),
    };
    assert!(draft.validate().is_err());
    draft.change(Field::Address, "192.168.1.20/33".to_owned());
    assert!(draft.validate().is_err());
    draft.change(Field::Address, "192.168.1.20/24".to_owned());
    draft.change(Field::Gateway, "inválido".to_owned());
    assert!(draft.validate().is_err());
    draft.change(Field::Gateway, "192.168.1.1".to_owned());
    draft.change(Field::Dns, "8.8.8.8, 1.1.1.1".to_owned());
    assert!(draft.validate().is_ok());
}

#[test]
fn gravacao_ipv4_preserva_campos_ipv6() {
    let draft = Draft {
        method: Method::Manual,
        address: "192.168.1.20/24".to_owned(),
        gateway: "192.168.1.1".to_owned(),
        dns: "1.1.1.1".to_owned(),
    };
    let args = draft.arguments("uuid").unwrap();
    assert_eq!(&args[..3], ["connection", "modify", "uuid"]);
    assert_eq!(args[3], "ipv4.method");
    assert!(args.iter().all(|argument| !argument.starts_with("ipv6.")));
}

#[test]
fn carrega_conexao_ativa_e_grava_ajustes_ipv4() {
    let details = detalhes("uuid-1");
    let address = "nmcli -g IP4.ADDRESS device show enp7s0";
    let gateway = "nmcli -g IP4.GATEWAY device show enp7s0";
    let mut commands = FakeCommands::new()
        .with(ATIVAS, "uuid-1:enp7s0\nuuid-loop:lo\n")
        .with(&details, "Conexão cabeada 1\nauto\n\n\n\n")
        .with(address, "192.168.0.70/24\n")
        .with(gateway, "192.168.0.1\n");
    let mut network = super::Network::load(&commands);
    assert_eq!(network.profiles.len(), 1);
    assert_eq!(network.draft.as_ref().unwrap().address, "192.168.0.70/24");
    let draft = network.draft.as_mut().unwrap();
    draft.method = Method::Manual;
    draft.change(Field::Dns, "1.1.1.1".to_owned());
    let args = draft.arguments("uuid-1").unwrap();
    let command = format!("nmcli {}", args.join(" "));
    commands = commands.accepting(&command);
    assert!(network.save(&commands).is_ok());
    assert!(commands.calls().contains(&command));
}

#[test]
fn nome_com_dois_pontos_aparece_sem_escape() {
    let commands = FakeCommands::new().with(ATIVAS, "uuid-1:wlan0\n").with(
        &detalhes("uuid-1"),
        "Casa: 5G\nmanual\n10.0.0.2/24\n10.0.0.1\n\n",
    );
    let network = super::Network::load(&commands);
    assert_eq!(network.profiles.len(), 1);
    assert_eq!(network.profiles[0].name, "Casa: 5G");
}

#[test]
fn endereco_atual_com_varios_valores_usa_o_primeiro() {
    let commands = FakeCommands::new()
        .with(ATIVAS, "uuid-1:enp7s0\n")
        .with(&detalhes("uuid-1"), "Cabo\nauto\n\n\n\n")
        .with(
            "nmcli -g IP4.ADDRESS device show enp7s0",
            "192.168.0.70/24 | 192.168.0.71/24\n",
        )
        .with("nmcli -g IP4.GATEWAY device show enp7s0", "\n");
    let network = super::Network::load(&commands);
    assert_eq!(network.profiles[0].address, "192.168.0.70/24");
    assert_eq!(network.profiles[0].gateway, "");
}

#[test]
fn metodos_ipv4_sem_edicao_nao_sao_oferecidos() {
    let commands = FakeCommands::new()
        .with(ATIVAS, "uuid-1:wlan0\nuuid-2:enp7s0\nuuid-3:ap0\n")
        .with(&detalhes("uuid-1"), "Só IPv6\ndisabled\n\n\n\n")
        .with(&detalhes("uuid-2"), "Cabo\nmanual\n10.0.0.2/24\n\n\n")
        .with(&detalhes("uuid-3"), "Hotspot\nshared\n\n\n\n");
    let network = super::Network::load(&commands);
    let uuids: Vec<_> = network.profiles.iter().map(|p| p.uuid.as_str()).collect();
    assert_eq!(uuids, ["uuid-2"]);
    assert_eq!(network.draft.unwrap().address, "10.0.0.2/24");
}

#[test]
fn conexao_selecionada_e_a_que_recebe_a_gravacao() {
    let segunda = Draft {
        method: Method::Manual,
        address: "10.0.0.2/24".to_owned(),
        gateway: "10.0.0.1".to_owned(),
        dns: String::new(),
    };
    let gravacao = format!("nmcli {}", segunda.arguments("uuid-2").unwrap().join(" "));
    let commands = FakeCommands::new()
        .with(ATIVAS, "uuid-1:wlan0\nuuid-2:enp7s0\n")
        .with(&detalhes("uuid-1"), "Wi-Fi\nmanual\n192.168.1.5/24\n\n\n")
        .with(
            &detalhes("uuid-2"),
            "Cabo\nmanual\n10.0.0.2/24\n10.0.0.1\n\n",
        )
        .accepting(&gravacao);
    let mut network = super::Network::load(&commands);
    network.select(1);
    assert_eq!(network.draft.as_ref(), Some(&segunda));
    network.select(9);
    assert_eq!(network.selected, 1);
    assert!(network.save(&commands).is_ok());
    assert!(commands.calls().contains(&gravacao));
    assert!(
        commands
            .calls()
            .iter()
            .all(|call| !call.contains("modify uuid-1"))
    );
}
