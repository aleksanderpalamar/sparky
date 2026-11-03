use crate::domain::system::idle::{chosen, label, row, stepped};
use crate::domain::system::{Control, RowId};
use crate::domain::value::Step;

#[test]
fn a_linha_mostra_o_tempo_e_as_opcoes() {
    let linha = row(10);
    assert_eq!(linha.id, RowId::IdleLock);
    assert_eq!(linha.label, "Bloquear por inatividade");
    assert_eq!(linha.value, "10 minutos");
    let Control::Choice(opcoes) = linha.control else {
        panic!("a linha deveria oferecer as opções");
    };
    assert_eq!(opcoes.first().map(String::as_str), Some("Nunca"));
    assert_eq!(opcoes.last().map(String::as_str), Some("1 hora"));
}

#[test]
fn os_nomes_seguem_o_portugues() {
    assert_eq!(label(0), "Nunca");
    assert_eq!(label(1), "1 minuto");
    assert_eq!(label(5), "5 minutos");
    assert_eq!(label(60), "1 hora");
    assert_eq!(label(90), "90 minutos");
}

#[test]
fn as_setas_andam_pelas_opcoes_sem_dar_a_volta() {
    assert_eq!(stepped(10, Step::More), 15);
    assert_eq!(stepped(10, Step::Less), 5);
    assert_eq!(stepped(0, Step::Less), 0);
    assert_eq!(stepped(60, Step::More), 60);
}

#[test]
fn um_tempo_fora_das_opcoes_vai_para_a_vizinha() {
    assert_eq!(stepped(7, Step::More), 10);
    assert_eq!(stepped(7, Step::Less), 5);
    assert_eq!(stepped(90, Step::Less), 60);
}

#[test]
fn escolher_pelo_nome_devolve_os_minutos() {
    assert_eq!(chosen("Nunca"), Some(0));
    assert_eq!(chosen(" 30 minutos "), Some(30));
    assert_eq!(chosen("1 hora"), Some(60));
    assert_eq!(chosen("toda hora"), None);
}
