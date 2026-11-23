use super::exec::{Fields, arguments};

const FIELDS: Fields<'static> = Fields {
    name: "Meu App",
    icon: Some("meu-app"),
    location: "/usr/share/applications/meu-app.desktop",
};

fn args(exec: &str) -> Option<Vec<String>> {
    arguments(exec, &FIELDS)
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
fn remove_os_codigos_de_arquivo_e_url_sem_arquivos_para_abrir() {
    assert_eq!(args("firefox %u"), list(&["firefox"]));
    assert_eq!(args("gimp %F --as-new"), list(&["gimp", "--as-new"]));
    assert_eq!(args("app %d %D %n %N %v %m"), list(&["app"]));
}

#[test]
fn expande_nome_local_e_icone_da_entrada() {
    assert_eq!(
        args("app --caption %c %U"),
        list(&["app", "--caption", "Meu App"])
    );
    assert_eq!(
        args("app --entry=%k"),
        list(&["app", "--entry=/usr/share/applications/meu-app.desktop"])
    );
    assert_eq!(args("app %i"), list(&["app", "--icon", "meu-app"]));
}

#[test]
fn sem_icone_o_codigo_do_icone_some_e_sem_local_vira_vazio() {
    let sem = Fields {
        name: "X",
        icon: None,
        location: "",
    };
    assert_eq!(arguments("app %i --x", &sem), list(&["app", "--x"]));
    assert_eq!(arguments("app %k", &sem), list(&["app", ""]));
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
