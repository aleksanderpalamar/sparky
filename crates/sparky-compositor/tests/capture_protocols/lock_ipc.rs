use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use sparky_ipc::{Event, Reply, Request, SessionAction};
use wayland_protocols::ext::session_lock::v1::client::{
    ext_session_lock_manager_v1 as lock_manager, ext_session_lock_v1 as lock,
};

use super::client::Client;
use super::fixture::Error;

const PRAZO: Duration = Duration::from_secs(3);

static RECEBIDO: AtomicBool = AtomicBool::new(false);

extern "C" fn recebeu(_: libc::c_int) {
    RECEBIDO.store(true, Ordering::SeqCst);
}

fn bloquear(client: &mut Client) -> Result<lock::ExtSessionLockV1, Error> {
    let qh = client.queue.handle();
    let manager: lock_manager::ExtSessionLockManagerV1 = client.bindings.bind(&qh, 1..=1, ())?;
    let session = manager.lock(&qh, ());
    client.await_event(|events| events.locked)?;
    Ok(session)
}

fn assinar(client: &Client) -> Result<Receiver<Event>, Error> {
    let events = client.ipc()?.subscribe()?;
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for event in events {
            if sender.send(event).is_err() {
                return;
            }
        }
    });
    Ok(receiver)
}

fn proximo_do_bloqueio(events: &Receiver<Event>) -> Result<Event, Error> {
    loop {
        match events.recv_timeout(PRAZO)? {
            Event::WorkspacesChanged => continue,
            event => return Ok(event),
        }
    }
}

#[test]
fn bloquear_e_desbloquear_avisam_quem_assina_o_ipc() -> Result<(), Error> {
    let mut client = Client::connect("ipc")?;
    let events = assinar(&client)?;
    let session = bloquear(&mut client)?;
    assert_eq!(proximo_do_bloqueio(&events)?, Event::Locked);
    session.unlock_and_destroy();
    client.roundtrip()?;
    assert_eq!(proximo_do_bloqueio(&events)?, Event::Unlocked);
    Ok(())
}

#[test]
fn o_pedido_de_desbloqueio_chega_ao_bloqueador_como_sigusr1() -> Result<(), Error> {
    unsafe {
        libc::signal(
            libc::SIGUSR1,
            recebeu as extern "C" fn(libc::c_int) as libc::sighandler_t,
        )
    };
    let mut client = Client::connect("ipc")?;
    let _session = bloquear(&mut client)?;
    let pedido = Request::Session {
        action: SessionAction::Unlock,
    };
    assert_eq!(client.ipc()?.request(&pedido)?, Reply::Done);
    let inicio = Instant::now();
    while !RECEBIDO.load(Ordering::SeqCst) && inicio.elapsed() < PRAZO {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(RECEBIDO.load(Ordering::SeqCst));
    Ok(())
}
