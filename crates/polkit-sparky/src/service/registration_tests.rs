use std::sync::Mutex;

use super::{AUTHORITY, AUTHORITY_PATH, PATH, registration, test_support::Bus};

#[derive(Default)]
struct Records {
    calls: Mutex<Vec<(String, String, String)>>,
}

#[zbus::interface(name = "org.freedesktop.PolicyKit1.Authority")]
impl Records {
    fn register_authentication_agent(
        &self,
        subject: registration::Subject,
        locale: &str,
        path: &str,
    ) {
        let id = subject
            .1
            .get("session-id")
            .and_then(|value| <&str>::try_from(value).ok());
        self.calls.lock().unwrap().push((
            subject.0,
            id.unwrap_or_default().to_owned(),
            path.to_owned(),
        ));
        assert!(!locale.is_empty());
    }

    fn unregister_authentication_agent(&self, subject: registration::Subject, path: &str) {
        self.calls
            .lock()
            .unwrap()
            .push((subject.0, "unregister".to_owned(), path.to_owned()));
    }
}

#[tokio::test]
async fn registra_e_remove_somente_a_sessao_informada() {
    let bus = Bus::start();
    let authority = zbus::connection::Builder::address(bus.address.as_str())
        .unwrap()
        .name(AUTHORITY)
        .unwrap()
        .serve_at(AUTHORITY_PATH, Records::default())
        .unwrap()
        .build()
        .await
        .unwrap();
    let client = bus.connection().await;
    let proxy = registration::authority(&client).await.unwrap();
    let subject = registration::subject("sessao-privada");
    registration::register(&proxy, &subject).await.unwrap();
    registration::unregister(&proxy, &subject).await.unwrap();
    let records = authority
        .object_server()
        .interface::<_, Records>(AUTHORITY_PATH)
        .await
        .unwrap();
    let records = records.get().await;
    assert_eq!(
        *records.calls.lock().unwrap(),
        vec![
            (
                "unix-session".to_owned(),
                "sessao-privada".to_owned(),
                PATH.to_owned()
            ),
            (
                "unix-session".to_owned(),
                "unregister".to_owned(),
                PATH.to_owned()
            ),
        ]
    );
}

struct Login;

#[zbus::interface(name = "org.freedesktop.login1.Manager")]
impl Login {
    #[zbus(name = "GetSessionByPID")]
    fn get_session_by_pid(&self, pid: u32) -> zbus::zvariant::OwnedObjectPath {
        assert_eq!(pid, std::process::id());
        "/org/freedesktop/login1/session/fixture"
            .try_into()
            .unwrap()
    }
}

struct Session;

#[zbus::interface(name = "org.freedesktop.login1.Session")]
impl Session {
    #[zbus(property)]
    fn id(&self) -> &str {
        "sessao-do-agente"
    }
}

#[tokio::test]
async fn encontra_a_sessao_logind_pelo_pid_do_proprio_agente() {
    let bus = Bus::start();
    let _login = zbus::connection::Builder::address(bus.address.as_str())
        .unwrap()
        .name("org.freedesktop.login1")
        .unwrap()
        .serve_at("/org/freedesktop/login1", Login)
        .unwrap()
        .serve_at("/org/freedesktop/login1/session/fixture", Session)
        .unwrap()
        .build()
        .await
        .unwrap();
    let client = bus.connection().await;
    assert_eq!(
        registration::session_id(&client).await.unwrap(),
        "sessao-do-agente"
    );
}
