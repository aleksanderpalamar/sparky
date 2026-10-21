use crate::domain::system::passwd;
use crate::domain::system::users::load;
use crate::ports::fake::FakeCommands;

const PASSWD: &str = "root:x:0:0::/root:/bin/sh\nana:x:1000:1000:Ana Silva,,,:/home/ana:/bin/zsh\nconvidado:x:1001:1001::/home/convidado:/bin/sh\nnobody:x:65534:65534::/:/sbin/nologin\nlixo\n";
const GROUP: &str = "wheel:x:998:ana\nana:x:1000:\nconvidado:x:1001:\n";

pub fn sistema() -> FakeCommands {
    FakeCommands::new().with("getent passwd", PASSWD).with("getent group", GROUP)
        .with("loginctl list-users --no-legend", "1000 ana yes active\n")
}

#[test]
fn le_contas_humanas_e_seus_campos_sem_root_ou_nobody() {
    let accounts = passwd::humans(PASSWD);
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].name, "ana");
    assert_eq!(accounts[0].uid, 1000);
    assert_eq!(accounts[0].full_name, "Ana Silva");
    assert_eq!(accounts[0].home, "/home/ana");
    assert_eq!(accounts[0].shell, "/bin/zsh");
    assert!(passwd::humans("x:x:errado:1000::/:/bin/sh\n").is_empty());
}

#[test]
fn grupos_incluem_o_primario_e_os_suplementares_sem_duplicacao() {
    let account = passwd::humans(PASSWD).remove(0);
    assert_eq!(passwd::groups_of(GROUP, &account), ["ana", "wheel"]);
}

#[test]
fn mostra_nome_tipo_e_sessao() {
    let users = load(&sistema());
    assert_eq!(users[0].title(), "Ana Silva (ana)");
    assert_eq!(users[0].role(), "administrador");
    assert_eq!(users[0].session(), "sessão aberta");
    assert_eq!(users[1].title(), "convidado");
    assert_eq!(users[1].role(), "comum");
    assert_eq!(users[1].session(), "sem sessão");
}

#[test]
fn grupo_sudo_tambem_identifica_administradores() {
    let commands = FakeCommands::new().with("getent passwd", PASSWD).with("getent group", "sudo:x:27:ana\n");
    assert_eq!(load(&commands)[0].role(), "administrador");
}

#[test]
fn indisponibilidade_nao_vira_afirmacao_sobre_sessoes_ou_grupos() {
    let users = load(&FakeCommands::new().with("getent passwd", PASSWD));
    assert_eq!(users[0].role(), "não disponível");
    assert_eq!(users[0].session(), "não disponível");
    assert!(load(&FakeCommands::new()).is_empty());
}
