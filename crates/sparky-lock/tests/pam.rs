use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use sparky_lock::adapters::pam::{PamAuthenticator, SERVICE};
use sparky_lock::ports::authenticator::Authenticator;
use sparky_login_core::protocol::Refusal;
use sparky_pam::LinuxPam;
use zeroize::Zeroizing;

const CONFERE: &str = "#!/bin/sh\nsenha=$(tr -d '\\000')\n[ \"$senha\" = \"segredo\" ]\n";

fn bloqueio() -> (tempfile::TempDir, PamAuthenticator) {
    let dir = tempfile::Builder::new()
        .prefix("bloqueio")
        .tempdir_in(env!("CARGO_TARGET_TMPDIR"))
        .unwrap_or_else(|error| panic!("{error}"));
    let script = dir.path().join("confere");
    fs::write(&script, CONFERE).unwrap_or_else(|error| panic!("{error}"));
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755))
        .unwrap_or_else(|error| panic!("{error}"));
    let servico = format!(
        "auth [success=1 ignore=1 default=ignore] pam_exec.so type=auth expose_authtok quiet {}\nauth requisite pam_deny.so\nauth required pam_permit.so\n",
        script.display()
    );
    fs::write(dir.path().join(SERVICE), servico).unwrap_or_else(|error| panic!("{error}"));
    let pam = LinuxPam {
        confdir: Some(PathBuf::from(dir.path())),
    };
    let user = nix::unistd::User::from_uid(nix::unistd::getuid())
        .ok()
        .flatten()
        .map(|user| user.name)
        .unwrap_or_default();
    (dir, PamAuthenticator::new(pam, user))
}

#[test]
fn a_senha_certa_desbloqueia_pelo_servico_sparky_lock() {
    let (_dir, autenticador) = bloqueio();
    assert_eq!(
        autenticador.unlock(Zeroizing::new("segredo".to_owned())),
        Ok(())
    );
}

#[test]
fn a_senha_errada_e_recusada() {
    let (_dir, autenticador) = bloqueio();
    assert_eq!(
        autenticador.unlock(Zeroizing::new("errada".to_owned())),
        Err(Refusal::Denied)
    );
}
