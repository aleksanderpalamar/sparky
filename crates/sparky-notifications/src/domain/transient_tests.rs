use super::{Center, Incoming, Urgency};

fn transient() -> Incoming {
    Incoming {
        sender: ":1.10".to_owned(),
        title: "Aviso temporário".to_owned(),
        transient: true,
        timeout: 5000,
        ..Default::default()
    }
}

#[test]
fn transitorio_chega_com_central_aberta_sem_sumir_ou_entrar_no_historico() {
    let mut center = Center::default();
    center.center_open(":1.20", true);
    let id = center.notify(transient(), 0).unwrap();
    assert_eq!(
        center
            .popups()
            .iter()
            .map(|card| card.id)
            .collect::<Vec<_>>(),
        [id]
    );
    assert!(center.snapshot().items.is_empty());
    assert_eq!(center.expire(5000), [(id, 1)]);
    center.center_open(":1.20", false);
    assert!(center.popups().is_empty());
    assert!(center.snapshot().items.is_empty());
}

#[test]
fn abrir_central_preserva_transitorio_existente_e_suprime_so_as_duplicatas() {
    let mut center = Center::default();
    let id = center.notify(transient(), 0).unwrap();
    let mut normal = transient();
    normal.transient = false;
    let archived = center.notify(normal, 1).unwrap();
    assert_eq!(center.popups().len(), 2);
    center.center_open(":1.20", true);
    center.center_open(":1.21", true);
    assert_eq!(
        center
            .popups()
            .iter()
            .map(|card| card.id)
            .collect::<Vec<_>>(),
        [id]
    );
    assert_eq!(center.snapshot().items[0].id, archived);
    center.center_open(":1.20", false);
    assert_eq!(center.popups().len(), 1);
    assert_eq!(center.expire(5000), [(id, 1)]);
    assert!(center.popups().is_empty());
}

#[test]
fn nao_perturbe_silencia_transitorios_com_ou_sem_central_aberta_sem_reexibir() {
    for (open, urgency) in [
        (false, Urgency::Normal),
        (true, Urgency::Normal),
        (false, Urgency::Critical),
        (true, Urgency::Critical),
    ] {
        let mut center = Center::default();
        center.center_open(":1.20", open);
        let mut input = transient();
        input.urgency = urgency;
        center.set_dnd(true);
        center.notify(input.clone(), 0).unwrap();
        assert!(center.popups().is_empty());
        center.set_dnd(false);
        assert!(center.popups().is_empty());
        center.notify(input, 1).unwrap();
        assert_eq!(center.popups().len(), 1);
        center.set_dnd(true);
        assert!(center.popups().is_empty());
        center.center_open(":1.21", true);
        center.set_dnd(false);
        assert!(center.popups().is_empty());
        assert!(center.snapshot().items.is_empty());
    }
}

#[test]
fn substituicao_atualiza_visibilidade_e_historico_conforme_hint_transient() {
    let mut center = Center::default();
    center.center_open(":1.20", true);
    let id = center.notify(transient(), 0).unwrap();
    let mut normal = transient();
    normal.replaces = id;
    normal.transient = false;
    center.notify(normal, 1).unwrap();
    assert!(center.popups().is_empty());
    assert_eq!(center.snapshot().items[0].id, id);
    let mut replacement = transient();
    replacement.replaces = id;
    center.notify(replacement, 2).unwrap();
    assert_eq!(center.popups()[0].id, id);
    assert!(center.snapshot().items.is_empty());
    center.center_open(":1.21", true);
    assert_eq!(center.popups()[0].id, id);
}
