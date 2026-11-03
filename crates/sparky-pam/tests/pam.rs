use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use sparky_login_core::protocol::Refusal;
use sparky_pam::LinuxPam;
use sparky_pam::{Pam, Transaction};
use tempfile::TempDir;
use zeroize::Zeroizing;

const SERVICO: &str = "sparky-login-teste";
const BLOQUEADO: &str = "sparky-login-bloqueado";
const SO_SENHA: &str = "sparky-so-senha";
const CONTA_NEGADA: &str = "sparky-conta-negada";
const AVISA: &str = "#!/bin/sh\necho conta bloqueada\nexit 1\n";
const CONFERE: &str = "#!/bin/sh\nsenha=$(tr -d '\\000')\n[ \"$senha\" = \"segredo\" ]\n";

fn usuario() -> String {
    nix::unistd::User::from_uid(nix::unistd::getuid())
        .ok()
        .flatten()
        .map(|user| user.name)
        .unwrap_or_default()
}

fn pasta() -> (TempDir, LinuxPam) {
    let dir = tempfile::Builder::new()
        .prefix("pam")
        .tempdir_in(env!("CARGO_TARGET_TMPDIR"))
        .unwrap_or_else(|error| panic!("{error}"));
    let script = dir.path().join("confere");
    escrever(&script, CONFERE);
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755))
        .unwrap_or_else(|error| panic!("{error}"));
    let servico = format!(
        "auth [success=1 ignore=1 default=ignore] pam_exec.so type=auth expose_authtok quiet {}\nauth requisite pam_deny.so\nauth required pam_permit.so\naccount required pam_permit.so\npassword required pam_deny.so\nsession required pam_permit.so\n",
        script.display()
    );
    escrever(&dir.path().join(SERVICO), &servico);
    let senha = format!(
        "auth [success=1 ignore=1 default=ignore] pam_exec.so type=auth expose_authtok quiet {}\nauth requisite pam_deny.so\nauth required pam_permit.so\n",
        script.display()
    );
    escrever(&dir.path().join(SO_SENHA), &senha);
    escrever(
        &dir.path().join(CONTA_NEGADA),
        &format!("{senha}account required pam_deny.so\n"),
    );
    let aviso = dir.path().join("avisa");
    escrever(&aviso, AVISA);
    fs::set_permissions(&aviso, fs::Permissions::from_mode(0o755))
        .unwrap_or_else(|error| panic!("{error}"));
    let bloqueado = format!(
        "auth [success=1 ignore=1 default=ignore] pam_exec.so type=auth stdout quiet {}\nauth requisite pam_deny.so\nauth required pam_permit.so\n",
        aviso.display()
    );
    escrever(&dir.path().join(BLOQUEADO), &bloqueado);
    let pam = LinuxPam {
        confdir: Some(PathBuf::from(dir.path())),
    };
    (dir, pam)
}

fn escrever(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

fn transacao(pam: &LinuxPam, senha: &str) -> Box<dyn Transaction> {
    pam.begin(SERVICO, &usuario(), Some(Zeroizing::new(senha.to_owned())))
        .unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn a_senha_chega_ao_modulo_pela_conversa_e_a_certa_entra() {
    let (_dir, pam) = pasta();
    assert_eq!(transacao(&pam, "segredo").authenticate(), Ok(()));
}

#[test]
fn a_senha_errada_e_recusada() {
    let (_dir, pam) = pasta();
    assert_eq!(
        transacao(&pam, "errada").authenticate(),
        Err(Refusal::Denied)
    );
}

#[test]
fn sem_senha_a_conversa_recusa_o_pedido_de_senha() {
    let (_dir, pam) = pasta();
    let mut transaction = pam
        .begin(SERVICO, &usuario(), None)
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(transaction.authenticate().is_err());
}

#[test]
fn a_sessao_abre_com_as_variaveis_e_o_terminal_e_fecha() {
    let (_dir, pam) = pasta();
    let mut transaction = transacao(&pam, "segredo");
    assert_eq!(transaction.authenticate(), Ok(()));
    assert_eq!(transaction.set_tty("/dev/tty1"), Ok(()));
    assert_eq!(transaction.put("XDG_SESSION_TYPE", "wayland"), Ok(()));
    assert_eq!(transaction.establish(), Ok(()));
    assert_eq!(transaction.open(), Ok(()));
    assert!(
        transaction
            .environment()
            .contains(&("XDG_SESSION_TYPE".to_owned(), "wayland".to_owned()))
    );
    transaction.close();
}

#[test]
fn servico_inexistente_nao_entra() {
    let (_dir, pam) = pasta();
    let transaction = pam.begin(
        "nao-existe",
        &usuario(),
        Some(Zeroizing::new("segredo".to_owned())),
    );
    if let Ok(mut transaction) = transaction {
        assert!(transaction.authenticate().is_err());
    }
}

#[test]
fn a_mensagem_do_pam_chega_na_recusa() {
    let (_dir, pam) = pasta();
    let mut transaction = pam
        .begin(
            BLOQUEADO,
            &usuario(),
            Some(Zeroizing::new("segredo".to_owned())),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        transaction.authenticate(),
        Err(Refusal::Unavailable("conta bloqueada".into()))
    );
}

#[test]
fn conferir_a_senha_nao_depende_das_regras_da_conta() {
    let (_dir, pam) = pasta();
    let mut transaction = pam
        .begin(
            SO_SENHA,
            &usuario(),
            Some(Zeroizing::new("segredo".to_owned())),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(transaction.authenticate(), Ok(()));
}

#[test]
fn as_regras_da_conta_recusam_mesmo_com_a_senha_certa() {
    let (_dir, pam) = pasta();
    let mut transaction = pam
        .begin(
            CONTA_NEGADA,
            &usuario(),
            Some(Zeroizing::new("segredo".to_owned())),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(transaction.authenticate(), Ok(()));
    assert_eq!(transaction.authorize(), Err(Refusal::Denied));
}

#[test]
fn a_conta_liberada_passa_nas_regras() {
    let (_dir, pam) = pasta();
    let mut transaction = transacao(&pam, "segredo");
    assert_eq!(transaction.authenticate(), Ok(()));
    assert_eq!(transaction.authorize(), Ok(()));
}
