use sparky_login_core::protocol::Refusal;

use crate::domain::job::{Order, Report};
use crate::worker::fakes::Launched;
use crate::worker::scene::{cena, entrada, login, par, rodar};

#[test]
fn senha_errada_e_recusada_sem_abrir_sessao() {
    let cena = cena(Err(Refusal::Denied));
    let (result, reports) = rodar(&cena, entrada(&login("palamar"), &[Order::Start]));
    assert_eq!(result, Ok(()));
    assert_eq!(
        reports,
        vec![Report::Refused {
            refusal: Refusal::Denied
        }]
    );
    assert_eq!(
        *cena.log.borrow(),
        vec!["begin sparky-login palamar segredo", "authenticate"]
    );
}

#[test]
fn senha_certa_espera_a_ordem_e_abre_a_sessao_pelo_shell_de_login() {
    let cena = cena(Ok(()));
    let (result, reports) = rodar(&cena, entrada(&login("palamar"), &[Order::Start]));
    assert_eq!(result, Ok(()));
    assert_eq!(
        reports,
        vec![Report::Authenticated, Report::Running, Report::Ended]
    );
    assert_eq!(
        *cena.log.borrow(),
        vec![
            "begin sparky-login palamar segredo",
            "authenticate",
            "authorize",
            "tty /dev/tty1",
            "establish",
            "open",
            "capture palamar /aparencia",
            "launch palamar",
            "wait",
            "capture palamar /aparencia",
            "close"
        ]
    );
    let launched = cena.launcher.launched.borrow();
    let Launched {
        command,
        environment,
        ..
    } = &launched[0];
    assert_eq!(
        command,
        &vec![
            "/usr/bin/zsh".to_owned(),
            "-l".to_owned(),
            "-c".to_owned(),
            "exec Hyprland".to_owned()
        ]
    );
    for esperado in [
        par("XDG_CURRENT_DESKTOP", "Hyprland"),
        par("XDG_SESSION_CLASS", "user"),
        par("XDG_RUNTIME_DIR", "/run/user/1000"),
        par("HOME", "/home/palamar"),
    ] {
        assert!(environment.contains(&esperado), "{esperado:?}");
    }
}

#[test]
fn sem_a_ordem_do_daemon_a_sessao_nao_abre() {
    let cena = cena(Ok(()));
    let (result, reports) = rodar(&cena, entrada(&login("palamar"), &[]));
    assert_eq!(result, Ok(()));
    assert_eq!(reports, vec![Report::Authenticated]);
    assert!(!cena.log.borrow().iter().any(|step| step == "open"));
}

#[test]
fn conta_sem_registro_local_e_recusada_depois_do_pam() {
    let cena = cena(Ok(()));
    let (_, reports) = rodar(&cena, entrada(&login("fantasma"), &[Order::Start]));
    let refusal = Refusal::Unavailable("a conta fantasma não tem registro neste computador".into());
    assert_eq!(reports, vec![Report::Refused { refusal }]);
    assert!(!cena.log.borrow().iter().any(|step| step == "open"));
}
