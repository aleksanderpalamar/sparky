use std::cmp::Ordering;

use crate::natural::compare;

fn ordered(mut names: Vec<&str>) -> Vec<&str> {
    names.sort_by(|left, right| compare(left, right));
    names
}

#[test]
fn numeros_sao_comparados_pelo_valor_e_nao_pelo_texto() {
    assert_eq!(
        ordered(vec!["foto10.png", "foto2.png", "foto1.png"]),
        ["foto1.png", "foto2.png", "foto10.png"]
    );
}

#[test]
fn maiusculas_e_minusculas_ficam_juntas() {
    assert_eq!(
        ordered(vec!["beta", "Alfa", "alfa2"]),
        ["Alfa", "alfa2", "beta"]
    );
}

#[test]
fn letras_acentuadas_seguem_a_letra_minuscula_correspondente() {
    assert_eq!(
        ordered(vec!["Zeta", "Élan", "alfa"]),
        ["alfa", "Élan", "Zeta"]
    );
    assert_eq!(compare("Ébano", "ébano 2"), Ordering::Less);
}

#[test]
fn zeros_a_esquerda_nao_mudam_o_valor_mas_desempatam() {
    assert_eq!(
        ordered(vec!["parte 010", "parte 9", "parte 10"]),
        ["parte 9", "parte 10", "parte 010"]
    );
}

#[test]
fn numeros_maiores_que_u64_continuam_em_ordem() {
    assert_eq!(
        ordered(vec![
            "n 100000000000000000000000",
            "n 99999999999999999999999"
        ]),
        ["n 99999999999999999999999", "n 100000000000000000000000"]
    );
}

#[test]
fn nomes_iguais_sem_caixa_desempatam_pelo_texto_original() {
    assert_eq!(compare("Nota", "nota"), Ordering::Less);
    assert_eq!(compare("nota", "nota"), Ordering::Equal);
}

#[test]
fn prefixo_vem_antes_do_nome_mais_longo() {
    assert_eq!(
        ordered(vec!["relatorio final", "relatorio"]),
        ["relatorio", "relatorio final"]
    );
}
