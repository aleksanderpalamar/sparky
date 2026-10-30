use super::{AutoDns, Draft, Field, Method};
use crate::domain::network::Lease;

fn manual(address: &str) -> Draft {
    Draft {
        method: Method::Manual,
        address: address.to_owned(),
        gateway: String::new(),
        primary_dns: String::new(),
        secondary_dns: String::new(),
        auto_dns: AutoDns::Use,
    }
}

fn valor<'a>(args: &'a [String], chave: &str) -> Option<&'a str> {
    let posicao = args.iter().position(|arg| arg == chave)?;
    args.get(posicao + 1).map(String::as_str)
}

#[test]
fn rejeita_endereco_prefixo_gateway_e_dns_invalidos() {
    let mut draft = manual("inválido");
    assert!(draft.validate().is_err());
    draft.change(Field::Address, "192.168.1.20/33".to_owned());
    assert!(draft.validate().is_err());
    draft.change(Field::Address, "192.168.1.20/24".to_owned());
    draft.change(Field::Gateway, "inválido".to_owned());
    assert!(draft.validate().is_err());
    draft.change(Field::Gateway, "192.168.1.1".to_owned());
    draft.change(Field::SecondaryDns, "8.8.8".to_owned());
    assert!(draft.validate().unwrap_err().contains("8.8.8"));
    draft.change(Field::SecondaryDns, "8.8.8.8".to_owned());
    assert!(draft.validate().is_ok());
}

#[test]
fn gravacao_ipv4_preserva_campos_ipv6() {
    let mut draft = manual("192.168.1.20/24");
    draft.change(Field::Gateway, "192.168.1.1".to_owned());
    draft.change(Field::PrimaryDns, "1.1.1.1".to_owned());
    let args = draft.arguments("uuid").unwrap();
    assert_eq!(&args[..3], ["connection", "modify", "uuid"]);
    assert_eq!(args[3], "ipv4.method");
    assert!(args.iter().all(|argument| !argument.starts_with("ipv6.")));
}

#[test]
fn perfil_com_mais_de_um_endereco_continua_gravavel() {
    let draft = manual("192.168.1.20/24,  10.0.0.2/8 ");
    let args = draft.arguments("uuid").unwrap();
    assert_eq!(
        valor(&args, "ipv4.addresses"),
        Some("192.168.1.20/24, 10.0.0.2/8")
    );
    let erro = manual("192.168.1.20/24, 10.0.0.2").validate().unwrap_err();
    assert!(erro.contains("10.0.0.2"));
    assert!(manual(" , ").validate().is_err());
}

#[test]
fn dns_primario_e_secundario_formam_a_lista_gravada() {
    let mut draft = manual("192.168.1.20/24");
    draft.change(Field::PrimaryDns, " 1.1.1.1 ".to_owned());
    draft.change(Field::SecondaryDns, "8.8.8.8".to_owned());
    let args = draft.arguments("uuid").unwrap();
    assert_eq!(valor(&args, "ipv4.dns"), Some("1.1.1.1 8.8.8.8"));
    draft.change(Field::PrimaryDns, String::new());
    let args = draft.arguments("uuid").unwrap();
    assert_eq!(valor(&args, "ipv4.dns"), Some("8.8.8.8"));
}

#[test]
fn servidores_do_perfil_ocupam_primario_e_secundario() {
    let mut draft = manual("192.168.1.20/24");
    draft.set_servers("1.1.1.1,8.8.8.8,9.9.9.9");
    assert_eq!(draft.primary_dns, "1.1.1.1");
    assert_eq!(draft.secondary_dns, "8.8.8.8, 9.9.9.9");
    let args = draft.arguments("uuid").unwrap();
    assert_eq!(valor(&args, "ipv4.dns"), Some("1.1.1.1 8.8.8.8 9.9.9.9"));
}

#[test]
fn trocar_para_manual_preenche_com_o_que_o_roteador_entregou() {
    let lease = Lease {
        address: "192.168.0.70/24".to_owned(),
        gateway: "192.168.0.1".to_owned(),
        dns: vec!["192.168.0.1".to_owned(), "8.8.8.8".to_owned()],
    };
    let mut draft = Draft {
        method: Method::Auto,
        ..manual("")
    };
    draft.switch(Method::Manual, &lease);
    assert_eq!(draft.method, Method::Manual);
    assert_eq!(draft.address, "192.168.0.70/24");
    assert_eq!(draft.gateway, "192.168.0.1");
    assert_eq!(draft.primary_dns, "192.168.0.1");
    assert_eq!(draft.secondary_dns, "8.8.8.8");
    draft.change(Field::Address, "192.168.0.50/24".to_owned());
    draft.switch(Method::Auto, &lease);
    draft.switch(Method::Manual, &lease);
    assert_eq!(draft.address, "192.168.0.50/24");
}

#[test]
fn dns_do_dhcp_e_uma_escolha_explicita() {
    let mut draft = Draft {
        method: Method::Auto,
        primary_dns: "1.1.1.1".to_owned(),
        ..manual("")
    };
    let args = draft.arguments("uuid").unwrap();
    assert_eq!(valor(&args, "ipv4.ignore-auto-dns"), Some("no"));
    assert_eq!(valor(&args, "ipv4.addresses"), Some(""));
    draft.auto_dns = AutoDns::Ignore;
    let args = draft.arguments("uuid").unwrap();
    assert_eq!(valor(&args, "ipv4.ignore-auto-dns"), Some("yes"));
    draft.change(Field::PrimaryDns, String::new());
    assert!(draft.validate().unwrap_err().contains("DNS"));
    draft.auto_dns = AutoDns::Use;
    assert!(draft.validate().is_ok());
}

#[test]
fn modo_manual_preserva_a_escolha_de_dns_do_perfil() {
    let mut draft = manual("192.168.1.20/24");
    draft.auto_dns = AutoDns::Ignore;
    let args = draft.arguments("uuid").unwrap();
    assert_eq!(valor(&args, "ipv4.ignore-auto-dns"), Some("yes"));
    assert_eq!(AutoDns::parse("yes"), Some(AutoDns::Ignore));
    assert_eq!(AutoDns::parse("no"), Some(AutoDns::Use));
    assert_eq!(AutoDns::parse(""), None);
}
