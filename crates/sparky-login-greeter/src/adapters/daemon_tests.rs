use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::thread;

use sparky_login_core::codec::{receive, send};
use sparky_login_core::protocol::{Power, Refusal, Reply, Request};
use zeroize::Zeroizing;

use crate::adapters::daemon::Daemon;
use crate::domain::form::Credentials;
use crate::ports::backend::Backend;

fn daemon_falso(respostas: Vec<Reply>) -> (Daemon, thread::JoinHandle<Vec<String>>) {
    let (ours, theirs) = UnixStream::pair().unwrap_or_else(|error| panic!("{error}"));
    let server = thread::spawn(move || {
        let mut writer = theirs.try_clone().unwrap_or_else(|error| panic!("{error}"));
        let mut reader = BufReader::new(theirs);
        let mut pedidos = Vec::new();
        for resposta in respostas {
            let Ok(Some(pedido)) = receive::<Request>(&mut reader) else {
                break;
            };
            pedidos.push(format!("{pedido:?}"));
            let _ = send(&mut writer, &resposta);
        }
        pedidos
    });
    (
        Daemon::from_stream(ours).unwrap_or_else(|error| panic!("{error}")),
        server,
    )
}

fn credenciais(senha: &str) -> Credentials {
    Credentials {
        username: "palamar".into(),
        password: Zeroizing::new(senha.into()),
    }
}

#[test]
fn ola_traz_o_ultimo_usuario_e_a_ultima_sessao() {
    let (daemon, _) = daemon_falso(vec![Reply::Welcome {
        last_user: Some("palamar".into()),
        last_session: Some("hyprland".into()),
    }]);
    assert_eq!(
        daemon.hello(),
        Ok((Some("palamar".into()), Some("hyprland".into())))
    );
}

#[test]
fn login_aceito_recusado_ou_com_falha_vira_resultado_da_tela() {
    let respostas = vec![
        Reply::LoggedIn,
        Reply::Refused {
            refusal: Refusal::Denied,
        },
        Reply::Failed {
            message: "já há um login em andamento".into(),
        },
        Reply::Done,
    ];
    let (daemon, server) = daemon_falso(respostas);
    assert_eq!(daemon.login(credenciais("segredo"), "sparky"), Ok(()));
    assert_eq!(
        daemon.login(credenciais("errada"), "sparky"),
        Err(Refusal::Denied)
    );
    assert_eq!(
        daemon.login(credenciais("segredo"), "sparky"),
        Err(Refusal::Unavailable("já há um login em andamento".into()))
    );
    assert_eq!(
        daemon.login(credenciais("segredo"), "sparky"),
        Err(Refusal::Unavailable(
            "o serviço de login respondeu fora de hora".into()
        ))
    );
    let pedidos = server.join().unwrap_or_default();
    assert!(pedidos[0].contains("palamar") && pedidos[0].contains("sparky"));
}

#[test]
fn energia_feita_ou_com_falha() {
    let (daemon, _) = daemon_falso(vec![
        Reply::Done,
        Reply::Failed {
            message: "sem permissão".into(),
        },
    ]);
    assert_eq!(daemon.power(Power::Reboot), Ok(()));
    assert_eq!(daemon.power(Power::Suspend), Err("sem permissão".into()));
}

#[test]
fn daemon_que_some_vira_falha_tecnica() {
    let (daemon, server) = daemon_falso(Vec::new());
    let _ = server.join();
    assert!(matches!(
        daemon.login(credenciais("segredo"), "sparky"),
        Err(Refusal::Unavailable(_))
    ));
    assert!(daemon.power(Power::Shutdown).is_err());
}
