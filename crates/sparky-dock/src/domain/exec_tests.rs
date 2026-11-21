use super::exec::arguments;

fn args(exec: &str) -> Option<Vec<String>> {
    arguments(exec)
}

fn list(words: &[&str]) -> Option<Vec<String>> {
    Some(words.iter().map(|word| (*word).to_owned()).collect())
}

#[test]
fn separa_o_programa_dos_argumentos_sem_shell() {
    assert_eq!(
        args("firefox --new-window"),
        list(&["firefox", "--new-window"])
    );
    assert_eq!(args("  code   --wait "), list(&["code", "--wait"]));
}

#[test]
fn remove_os_codigos_de_arquivo_e_url() {
    assert_eq!(args("firefox %u"), list(&["firefox"]));
    assert_eq!(args("gimp %F --as-new"), list(&["gimp", "--as-new"]));
    assert_eq!(args("app --file=%f"), list(&["app", "--file="]));
}

#[test]
fn porcentagem_dupla_vira_literal() {
    assert_eq!(args("echo 100%%"), list(&["echo", "100%"]));
}

#[test]
fn aspas_mantem_espacos_e_tratam_escapes() {
    assert_eq!(
        args(r#""/opt/My App/run" --name "a \"b\" \$c""#),
        list(&["/opt/My App/run", "--name", r#"a "b" $c"#])
    );
}

#[test]
fn metacaracteres_do_shell_viram_argumentos_comuns() {
    assert_eq!(
        args("app ; rm -rf ~"),
        list(&["app", ";", "rm", "-rf", "~"])
    );
}

#[test]
fn linha_invalida_ou_vazia_nao_gera_comando() {
    assert_eq!(args(r#""sem fim"#), None);
    assert_eq!(args("   "), None);
    assert_eq!(args("%U"), None);
}
