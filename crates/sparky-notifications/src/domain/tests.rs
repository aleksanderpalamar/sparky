use super::{Action, Center, Incoming, Urgency};

fn notification() -> Incoming {
    Incoming {
        sender: ":1.10".to_owned(),
        app: "Aplicativo".to_owned(),
        title: "Mensagem".to_owned(),
        body: "Texto".to_owned(),
        timeout: 5000,
        ..Default::default()
    }
}

#[test]
fn nao_perturbe_suprime_criticos_preserva_historico_e_nao_reexibe_avisos_antigos() {
    let mut center = Center::default();
    center.set_dnd(true);
    let mut input = notification();
    input.urgency = Urgency::Critical;
    let id = center.notify(input, 1000).unwrap();
    assert!(center.popups().is_empty());
    assert_eq!(center.snapshot().items[0].id, id);
    center.set_dnd(false);
    assert!(center.popups().is_empty());
    center.notify(notification(), 2000).unwrap();
    assert_eq!(center.popups().len(), 1);
}

#[test]
fn expirar_remove_id_e_acoes_mas_conserva_texto_no_historico() {
    let mut center = Center::default();
    let mut input = notification();
    input.actions = vec![Action {
        key: "default".to_owned(),
        label: "Abrir".to_owned(),
    }];
    let id = center.notify(input, 1000).unwrap();
    assert!(center.action(id, "default").is_some());
    assert_eq!(center.expire(6000), vec![(id, 1)]);
    assert!(center.action(id, "default").is_none());
    assert!(center.snapshot().items[0].notification.actions.is_empty());
    assert_eq!(center.snapshot().items[0].notification.body, "Texto");
}

#[test]
fn substituicao_atualiza_mesmo_id_e_respeita_o_remetente() {
    let mut center = Center::default();
    let id = center.notify(notification(), 0).unwrap();
    let mut input = notification();
    input.replaces = id;
    input.title = "Nova".to_owned();
    assert_eq!(center.notify(input, 100).unwrap(), id);
    assert_eq!(center.snapshot().items.len(), 1);
    assert_eq!(center.snapshot().items[0].notification.title, "Nova");
    let mut other = notification();
    other.sender = ":1.11".to_owned();
    other.replaces = id;
    assert!(center.notify(other, 200).is_err());
}

#[test]
fn transitorios_nao_entram_no_historico_e_a_fila_tem_limite() {
    let mut center = Center::default();
    let mut input = notification();
    input.transient = true;
    center.notify(input, 0).unwrap();
    assert!(center.snapshot().items.is_empty());
    for now in 1..=105 {
        center.notify(notification(), now).unwrap();
    }
    assert_eq!(center.snapshot().items.len(), 100);
    assert!(center.popups().len() <= 3);
}

#[test]
fn fechamento_do_cliente_e_limpeza_removem_conteudo_e_emitem_razao_correta() {
    let mut center = Center::default();
    let id = center.notify(notification(), 0).unwrap();
    assert!(center.close_client(id, ":1.99").is_err());
    assert_eq!(center.close_client(id, ":1.10").unwrap(), (id, 3));
    assert!(center.snapshot().items.is_empty());
    let id = center.notify(notification(), 1).unwrap();
    assert_eq!(center.clear(), vec![(id, 2)]);
    assert!(center.snapshot().items.is_empty());
}

#[test]
fn fechar_o_remetente_nao_faz_notify_send_sumir_antes_do_prazo() {
    let mut center = Center::default();
    let mut input = notification();
    input.actions = vec![Action {
        key: "default".to_owned(),
        label: "Abrir".to_owned(),
    }];
    let id = center.notify(input, 0).unwrap();
    assert!(center.owner_lost(":1.10").is_empty());
    assert_eq!(center.popups().len(), 1);
    assert!(center.action(id, "default").is_none());
    assert!(center.snapshot().items[0].notification.actions.is_empty());
    assert_eq!(center.expire(5000), vec![(id, 1)]);
}

#[test]
fn criticos_e_timeout_zero_nao_expiram_e_ativos_possuem_limite() {
    let mut center = Center::default();
    let mut critical = notification();
    critical.urgency = Urgency::Critical;
    center.notify(critical, 0).unwrap();
    let mut persistent = notification();
    persistent.timeout = 0;
    center.notify(persistent, 0).unwrap();
    assert!(center.expire(u64::MAX).is_empty());
    for now in 2..128 {
        center.notify(notification(), now).unwrap();
    }
    assert_eq!(center.notify(notification(), 129), Err(super::Error::Full));
}

#[test]
fn invocar_acao_remove_id_atomicamente_e_preserva_notificacao_residente() {
    let mut center = Center::default();
    let mut input = notification();
    input.actions = vec![Action {
        key: "open".to_owned(),
        label: "Abrir".to_owned(),
    }];
    let id = center.notify(input.clone(), 0).unwrap();
    assert_eq!(center.invoke(id, "wrong"), Err(super::Error::Unknown));
    assert_eq!(center.invoke(id, "open").unwrap(), Some((id, 2)));
    assert!(center.action(id, "open").is_none());
    input.resident = true;
    let id = center.notify(input, 1).unwrap();
    assert_eq!(center.invoke(id, "open").unwrap(), None);
    assert!(center.action(id, "open").is_some());
}
