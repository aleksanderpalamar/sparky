use super::tabs::{Closed, Tabs};

fn tres() -> Tabs<&'static str> {
    let mut abas = Tabs::default();
    abas.open("a");
    abas.open("b");
    abas
}

#[test]
fn abrir_guarda_a_atual_e_vai_para_a_nova_ao_lado() {
    let abas = tres();
    assert_eq!((abas.len(), abas.active()), (3, 2));
    assert_eq!(abas.stored(0), Some(&"a"));
    assert_eq!(abas.stored(1), Some(&"b"));
    assert_eq!(abas.stored(2), None);
}

#[test]
fn trocar_devolve_a_escolhida_e_guarda_a_atual() {
    let mut abas = tres();
    assert_eq!(abas.switch(0, "c"), Ok("a"));
    assert_eq!(abas.active(), 0);
    assert_eq!(abas.stored(2), Some(&"c"));
    assert_eq!(abas.switch(0, "a"), Err("a"));
    assert_eq!(abas.switch(9, "a"), Err("a"));
}

#[test]
fn fechar_a_atual_vai_para_a_vizinha_e_a_ultima_nunca_fecha() {
    let mut abas = tres();
    assert_eq!(abas.close(2), Closed::Current("b"));
    assert_eq!((abas.len(), abas.active()), (2, 1));
    assert_eq!(abas.close(0), Closed::Other);
    assert_eq!((abas.len(), abas.active()), (1, 0));
    assert_eq!(abas.close(0), Closed::Kept);
    assert_eq!(abas.len(), 1);
}

#[test]
fn fechar_uma_aba_antes_da_atual_mantem_a_atual() {
    let mut abas = tres();
    let _ = abas.switch(1, "c");
    assert_eq!(abas.close(0), Closed::Other);
    assert_eq!(abas.active(), 0);
    assert_eq!(abas.stored(1), Some(&"c"));
}

#[test]
fn achar_uma_aba_guardada_permite_atualizar_ela() {
    let mut abas = tres();
    if let Some(aba) = abas.find_mut(|aba| *aba == "a") {
        *aba = "z";
    }
    assert_eq!(abas.stored(0), Some(&"z"));
}
