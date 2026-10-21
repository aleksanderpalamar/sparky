use crate::domain::form::{Field, Form, Problem, Step};
use crate::domain::mode::Mode;

fn pronto(passo: Step) -> Option<String> {
    match passo {
        Step::Done(senha) => Some(senha.to_string()),
        _ => None,
    }
}

#[test]
fn destrancar_entrega_a_senha_digitada() {
    let mut form = Form::new(Mode::Unlock);
    form.type_in(Field::Password, "segredo".to_owned());
    assert_eq!(pronto(form.enter(Field::Password)).as_deref(), Some("segredo"));
}

#[test]
fn senha_vazia_nao_sai_da_janela() {
    let mut form = Form::new(Mode::Unlock);
    assert_eq!(form.enter(Field::Password), Step::Stay);
    assert_eq!(form.problem, Some(Problem::Empty));
    form.type_in(Field::Password, "x".to_owned());
    assert_eq!(form.problem, None);
}

#[test]
fn criar_pede_a_confirmacao_antes_de_entregar() {
    let mut form = Form::new(Mode::Create);
    form.type_in(Field::Password, "nova".to_owned());
    assert_eq!(form.enter(Field::Password), Step::Focus(Field::Confirmation));
    form.type_in(Field::Confirmation, "nova".to_owned());
    assert_eq!(pronto(form.enter(Field::Confirmation)).as_deref(), Some("nova"));
}

#[test]
fn confirmacao_diferente_e_apagada_com_aviso() {
    let mut form = Form::new(Mode::Create);
    form.type_in(Field::Password, "nova".to_owned());
    form.type_in(Field::Confirmation, "outra".to_owned());
    assert_eq!(form.enter(Field::Confirmation), Step::Stay);
    assert_eq!(form.problem, Some(Problem::Mismatch));
    assert_eq!(form.confirmation(), "");
}

#[test]
fn confirmar_sem_senha_volta_para_a_senha() {
    let mut form = Form::new(Mode::Create);
    form.type_in(Field::Confirmation, "x".to_owned());
    assert_eq!(form.enter(Field::Confirmation), Step::Focus(Field::Password));
    assert_eq!(form.problem, Some(Problem::Empty));
}

#[test]
fn tab_alterna_os_campos_so_ao_criar() {
    let mut criar = Form::new(Mode::Create);
    assert_eq!(criar.switch(), Some(Field::Confirmation));
    assert_eq!(criar.switch(), Some(Field::Password));
    assert_eq!(Form::new(Mode::Unlock).switch(), None);
}

#[test]
fn os_avisos_explicam_o_problema() {
    assert_eq!(Problem::Empty.message(), "A senha não pode ficar vazia.");
    assert_eq!(Problem::Mismatch.message(), "As senhas não conferem.");
}
