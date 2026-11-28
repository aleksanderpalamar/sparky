use super::history::History;

fn percurso() -> History<&'static str> {
    let mut history = History::default();
    history.visit("A");
    history.visit("B");
    history
}

#[test]
fn historico_novo_nao_tem_anterior_nem_proximo() {
    let history: History<&str> = History::default();
    assert_eq!(history.previous(), None);
    assert_eq!(history.next(), None);
}

#[test]
fn voltar_oferece_a_pasta_anterior_e_guarda_a_atual_para_avancar() {
    let mut history = percurso();
    assert_eq!(history.previous(), Some(&"B"));
    history.went_back("C");
    assert_eq!(history.previous(), Some(&"A"));
    assert_eq!(history.next(), Some(&"C"));
}

#[test]
fn avancar_desfaz_o_voltar() {
    let mut history = percurso();
    history.went_back("C");
    history.went_forward("B");
    assert_eq!(history.previous(), Some(&"B"));
    assert_eq!(history.next(), None);
}

#[test]
fn abrir_outra_pasta_depois_de_voltar_descarta_o_avanco() {
    let mut history = percurso();
    history.went_back("C");
    history.visit("B");
    assert_eq!(history.next(), None);
    assert_eq!(history.previous(), Some(&"B"));
}

#[test]
fn voltar_sem_anterior_nao_cria_avanco() {
    let mut history: History<&str> = History::default();
    history.went_back("A");
    history.went_forward("A");
    assert_eq!(history.previous(), None);
    assert_eq!(history.next(), None);
}
