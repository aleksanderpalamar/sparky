use std::path::PathBuf;

use super::socket::path_from;

fn text(value: &str) -> Option<String> {
    Some(value.to_owned())
}

#[test]
fn fica_no_diretorio_de_execucao_do_usuario() {
    assert_eq!(
        path_from(None, text("/run/user/1000")),
        Some(PathBuf::from("/run/user/1000/sparky-resources.sock"))
    );
}

#[test]
fn a_variavel_propria_tem_prioridade() {
    assert_eq!(
        path_from(text("/tmp/teste.sock"), text("/run/user/1000")),
        Some(PathBuf::from("/tmp/teste.sock"))
    );
    assert_eq!(
        path_from(text("  "), text("/run/user/1000")),
        Some(PathBuf::from("/run/user/1000/sparky-resources.sock"))
    );
}

#[test]
fn sem_diretorio_absoluto_nao_ha_socket() {
    assert_eq!(path_from(None, None), None);
    assert_eq!(path_from(None, text("relativo")), None);
}
