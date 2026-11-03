use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::mpsc::{self, Sender};

use crate::app::{Event, GRACE, run};
use crate::domain::guard::{Input, Screen};
use crate::domain::inhibitions::Hold;
use crate::ports::{Compositor, Logind};

#[derive(Default)]
struct Registro {
    chamadas: Mutex<Vec<String>>,
}

struct Inibidor(Arc<Registro>);

impl Drop for Inibidor {
    fn drop(&mut self) {
        self.0.anotar("soltar");
    }
}

impl Registro {
    fn anotar(&self, chamada: &str) {
        if let Ok(mut chamadas) = self.chamadas.lock() {
            chamadas.push(chamada.to_owned());
        }
    }

    fn chamadas(&self) -> Vec<String> {
        self.chamadas.lock().map(|c| c.clone()).unwrap_or_default()
    }

    fn inibidor_solto(&self) -> bool {
        self.chamadas().iter().any(|chamada| chamada == "soltar")
    }
}

struct LogindFalso(Arc<Registro>);
struct CompositorFalso(Arc<Registro>);

impl Logind for LogindFalso {
    type Inhibitor = Inibidor;

    async fn inhibit(&self) -> Result<Inibidor, String> {
        self.0.anotar("inibir");
        Ok(Inibidor(Arc::clone(&self.0)))
    }

    async fn hint(&self, screen: Screen) -> Result<(), String> {
        self.0.anotar(&format!("dica {screen:?}"));
        Ok(())
    }
}

impl Compositor for CompositorFalso {
    fn lock(&self) -> Result<(), String> {
        self.0.anotar("bloquear");
        Ok(())
    }

    fn unlock(&self) -> Result<(), String> {
        self.0.anotar("desbloquear");
        Ok(())
    }

    fn idle(&self, hold: Hold) -> Result<(), String> {
        self.0.anotar(&format!("tela acesa {hold:?}"));
        Ok(())
    }
}

fn iniciar() -> (Arc<Registro>, Sender<Event>, tokio::task::JoinHandle<()>) {
    let registro = Arc::new(Registro::default());
    let (sender, receiver) = mpsc::channel(8);
    let logind = LogindFalso(Arc::clone(&registro));
    let compositor = CompositorFalso(Arc::clone(&registro));
    let tarefa = tokio::spawn(async move { run(&logind, &compositor, receiver).await });
    (registro, sender, tarefa)
}

async fn enviar(sender: &Sender<Event>, input: Input) {
    let _ = sender.send(Event::Seen(input)).await;
    tokio::time::sleep(Duration::from_millis(10)).await;
}

#[tokio::test(start_paused = true)]
async fn comeca_segurando_a_suspensao_com_a_tela_aberta() {
    let (registro, _sender, _tarefa) = iniciar();
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(registro.chamadas(), ["inibir", "dica Open"]);
    assert!(!registro.inibidor_solto());
}

#[tokio::test(start_paused = true)]
async fn antes_de_suspender_bloqueia_e_solta_o_inibidor_so_depois_de_bloqueado() {
    let (registro, sender, _tarefa) = iniciar();
    enviar(&sender, Input::Sleeping).await;
    assert_eq!(
        registro.chamadas().last().map(String::as_str),
        Some("bloquear")
    );
    assert!(!registro.inibidor_solto());
    enviar(&sender, Input::Locked).await;
    assert!(registro.inibidor_solto());
    assert!(registro.chamadas().contains(&"dica Locked".to_owned()));
}

#[tokio::test(start_paused = true)]
async fn se_o_bloqueio_nao_chega_o_inibidor_e_solto_no_prazo() {
    let (registro, sender, _tarefa) = iniciar();
    enviar(&sender, Input::Sleeping).await;
    assert!(!registro.inibidor_solto());
    tokio::time::sleep(GRACE).await;
    assert!(registro.inibidor_solto());
}

#[tokio::test(start_paused = true)]
async fn os_pedidos_do_logind_chegam_ao_compositor() {
    let (registro, sender, _tarefa) = iniciar();
    enviar(&sender, Input::LockAsked).await;
    enviar(&sender, Input::UnlockAsked).await;
    assert_eq!(
        registro.chamadas(),
        ["inibir", "dica Open", "bloquear", "desbloquear"]
    );
}

#[tokio::test(start_paused = true)]
async fn o_fim_do_compositor_encerra_o_daemon() {
    let (_registro, sender, tarefa) = iniciar();
    let _ = sender.send(Event::CompositorGone).await;
    assert!(
        tokio::time::timeout(Duration::from_secs(1), tarefa)
            .await
            .is_ok()
    );
}

#[tokio::test(start_paused = true)]
async fn o_pedido_de_tela_acesa_chega_ao_compositor() {
    let (registro, sender, _tarefa) = iniciar();
    enviar(&sender, Input::Idle(Hold::Held)).await;
    enviar(&sender, Input::Idle(Hold::Free)).await;
    let chamadas = registro.chamadas();
    assert_eq!(&chamadas[2..], ["tela acesa Held", "tela acesa Free"]);
}
