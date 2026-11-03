use std::path::PathBuf;

use crate::domain::job::{Job, Report};
use crate::worker::fakes::Launched;
use crate::worker::scene::{cena, entrada, par, rodar};

#[test]
fn a_tela_abre_sessao_de_greeter_roda_o_comando_e_fecha_no_fim() {
    let cena = cena(Ok(()));
    let job = Job::Greeter {
        user: "sparky-login".into(),
        command: vec!["sparky".into(), "--login-screen".into()],
        vt: 1,
        socket: PathBuf::from("/run/sparky-login/greeter.sock"),
    };
    let (result, reports) = rodar(&cena, entrada(&job, &[]));
    assert_eq!(result, Ok(()));
    assert_eq!(reports, vec![Report::Running, Report::Ended]);
    assert_eq!(
        *cena.log.borrow(),
        vec![
            "begin sparky-login-greeter sparky-login -",
            "tty /dev/tty1",
            "establish",
            "open",
            "launch sparky-login",
            "wait",
            "close"
        ]
    );
    let launched = cena.launcher.launched.borrow();
    let Launched {
        command,
        environment,
        tty,
    } = &launched[0];
    assert_eq!(
        command,
        &vec!["sparky".to_owned(), "--login-screen".to_owned()]
    );
    assert!(environment.contains(&par(
        "SPARKY_LOGIN_SOCKET",
        "/run/sparky-login/greeter.sock"
    )));
    assert!(environment.contains(&par("XDG_SESSION_CLASS", "greeter")));
    assert_eq!(tty, &PathBuf::from("/dev/tty1"));
}

#[test]
fn sem_tarefa_e_erro() {
    let cena = cena(Ok(()));
    let (result, reports) = rodar(&cena, Vec::new());
    assert!(result.is_err());
    assert!(reports.is_empty());
}
