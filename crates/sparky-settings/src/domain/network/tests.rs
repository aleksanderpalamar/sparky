use super::{AutoDns, Draft, Method};
use crate::ports::fake::FakeCommands;

const ATIVAS: &str = "nmcli -t -f UUID,DEVICE connection show --active";

fn detalhes(uuid: &str) -> String {
    format!(
        "nmcli -e no -g connection.id,ipv4.method,ipv4.addresses,ipv4.gateway,ipv4.dns,ipv4.ignore-auto-dns connection show {uuid}"
    )
}

fn concessao(interface: &str) -> String {
    format!("nmcli -g IP4.ADDRESS,IP4.GATEWAY,IP4.DNS device show {interface}")
}

#[test]
fn dhcp_mostra_o_que_o_roteador_entregou_e_manual_parte_dele() {
    let mut commands = FakeCommands::new()
        .with(ATIVAS, "uuid-1:enp7s0\nuuid-loop:lo\n")
        .with(&detalhes("uuid-1"), "Conexão cabeada 1\nauto\n\n\n\nno\n")
        .with(
            &concessao("enp7s0"),
            "192.168.0.70/24 | 192.168.0.71/24\n192.168.0.1\n192.168.0.1\n",
        );
    let mut network = super::Network::load(&commands);
    assert_eq!(network.profiles.len(), 1);
    let lease = &network.profiles[0].lease;
    assert_eq!(lease.address, "192.168.0.70/24");
    assert_eq!(lease.gateway, "192.168.0.1");
    assert_eq!(lease.dns, ["192.168.0.1"]);
    assert_eq!(network.draft.as_ref().unwrap().address, "");
    network.switch(Method::Manual);
    let draft = network.draft.as_ref().unwrap();
    assert_eq!(draft.address, "192.168.0.70/24");
    assert_eq!(draft.primary_dns, "192.168.0.1");
    let command = format!("nmcli {}", draft.arguments("uuid-1").unwrap().join(" "));
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
fn concessao_indisponivel_fica_vazia() {
    let commands = FakeCommands::new()
        .with(ATIVAS, "uuid-1:enp7s0\n")
        .with(&detalhes("uuid-1"), "Cabo\nauto\n\n\n\n");
    let network = super::Network::load(&commands);
    let lease = &network.profiles[0].lease;
    assert_eq!(lease.address, "");
    assert_eq!(lease.gateway, "");
    assert!(lease.dns.is_empty());
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
        primary_dns: "1.1.1.1".to_owned(),
        secondary_dns: String::new(),
        auto_dns: AutoDns::Use,
    };
    let gravacao = format!("nmcli {}", segunda.arguments("uuid-2").unwrap().join(" "));
    let commands = FakeCommands::new()
        .with(ATIVAS, "uuid-1:wlan0\nuuid-2:enp7s0\n")
        .with(&detalhes("uuid-1"), "Wi-Fi\nmanual\n192.168.1.5/24\n\n\n")
        .with(
            &detalhes("uuid-2"),
            "Cabo\nmanual\n10.0.0.2/24\n10.0.0.1\n1.1.1.1\n",
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

#[test]
fn le_a_escolha_de_dns_automatico_do_perfil() {
    let commands = FakeCommands::new().with(ATIVAS, "uuid-1:enp7s0\n").with(
        &detalhes("uuid-1"),
        "Cabo\nmanual\n10.0.0.2/24\n\n1.1.1.1,8.8.8.8\nyes\n",
    );
    let network = super::Network::load(&commands);
    let draft = network.draft.unwrap();
    assert_eq!(draft.auto_dns, AutoDns::Ignore);
    assert_eq!(draft.primary_dns, "1.1.1.1");
    assert_eq!(draft.secondary_dns, "8.8.8.8");
}
