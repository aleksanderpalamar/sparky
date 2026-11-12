use super::batch_name::{Pattern, Problem, check, parse, plan};

fn nomes(lista: &[&str]) -> Vec<String> {
    lista.iter().map(|nome| (*nome).to_owned()).collect()
}

fn pares(lista: &[(&str, &str)]) -> Vec<(String, String)> {
    lista
        .iter()
        .map(|(de, para)| ((*de).to_owned(), (*para).to_owned()))
        .collect()
}

#[test]
fn padrao_com_n_numera_na_ordem_e_mantem_a_extensao() {
    let padrao = parse("Viagem {n}");
    assert_eq!(padrao, Some(Pattern::Numbered("Viagem {n}".to_owned())));
    let plano = plan(
        &nomes(&["b.JPG", "a.png", "sem-extensao"]),
        &padrao.unwrap_or(Pattern::Numbered(String::new())),
    );
    assert_eq!(
        plano,
        pares(&[
            ("b.JPG", "Viagem 1.JPG"),
            ("a.png", "Viagem 2.png"),
            ("sem-extensao", "Viagem 3"),
        ])
    );
}

#[test]
fn numeros_ganham_zeros_pela_quantidade_de_itens() {
    let itens: Vec<String> = (0..12).map(|i| format!("f{i}.txt")).collect();
    let plano = plan(&itens, &Pattern::Numbered("{n} - foto".to_owned()));
    assert_eq!(
        plano.first().map(|(_, novo)| novo.as_str()),
        Some("01 - foto.txt")
    );
    assert_eq!(
        plano.last().map(|(_, novo)| novo.as_str()),
        Some("12 - foto.txt")
    );
}

#[test]
fn seta_troca_um_trecho_em_todos_os_nomes() {
    let padrao = parse("IMG_ -> Praia ");
    assert_eq!(
        padrao,
        Some(Pattern::Replace {
            from: "IMG_".to_owned(),
            to: "Praia".to_owned(),
        })
    );
    let Some(padrao) = padrao else { return };
    assert_eq!(
        plan(&nomes(&["IMG_01.jpg", "IMG_02.jpg", "outro.jpg"]), &padrao),
        pares(&[
            ("IMG_01.jpg", "Praia01.jpg"),
            ("IMG_02.jpg", "Praia02.jpg"),
            ("outro.jpg", "outro.jpg"),
        ])
    );
}

#[test]
fn entrada_sem_padrao_reconhecido_e_recusada() {
    assert_eq!(parse(""), None);
    assert_eq!(parse("nome fixo"), None);
    assert_eq!(parse(" -> novo"), None);
}

#[test]
fn verifica_nomes_invalidos_repetidos_e_ja_ocupados() {
    let livre = |_: &str| false;
    assert_eq!(
        check(&pares(&[("a", "b/c")]), livre),
        Err(Problem::Invalid("b/c".to_owned()))
    );
    assert_eq!(
        check(&pares(&[("a", "x"), ("b", "x")]), livre),
        Err(Problem::Repeated("x".to_owned()))
    );
    let ocupado = |nome: &str| nome == "x" || nome == "b";
    assert_eq!(
        check(&pares(&[("a", "x")]), ocupado),
        Err(Problem::Taken("x".to_owned()))
    );
    assert_eq!(check(&pares(&[("a", "b"), ("b", "a")]), ocupado), Ok(()));
    assert_eq!(check(&pares(&[("a", "a")]), livre), Err(Problem::Unchanged));
}
