use std::future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gio::glib;

use super::{bounded, failure};

struct Dropped(Arc<AtomicBool>);

impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[test]
fn tempo_limite_encerra_a_espera_e_descarta_a_operacao_pendente() {
    let cancelled = Arc::new(AtomicBool::new(false));
    let guard = Dropped(cancelled.clone());
    let pending = async move {
        let _guard = guard;
        future::pending::<()>().await;
        Ok(())
    };
    let context = glib::MainContext::new();
    let result = context.block_on(bounded(pending, Duration::from_millis(10)));
    assert!(result.is_err_and(|error| error.matches(gio::IOErrorEnum::TimedOut)));
    assert!(cancelled.load(Ordering::SeqCst));
}

#[test]
fn resposta_imediata_preserva_sucesso_e_erro() {
    let context = glib::MainContext::new();
    assert_eq!(
        context.block_on(bounded(async { Ok(7) }, Duration::from_secs(1))),
        Ok(7)
    );
    let error = glib::Error::new(gio::IOErrorEnum::Busy, "ocupado");
    let result = context.block_on(bounded(
        future::ready(Err::<(), _>(error)),
        Duration::from_secs(1),
    ));
    assert!(result.is_err_and(|error| error.matches(gio::IOErrorEnum::Busy)));
}

#[test]
fn falhas_explicam_backend_ausente_dispositivo_ocupado_e_cancelamento() {
    let message = |kind| failure(glib::Error::new(kind, "teste"));
    assert!(message(gio::IOErrorEnum::NotSupported).contains("gvfs-mtp"));
    assert!(message(gio::IOErrorEnum::Busy).contains("outro aplicativo"));
    assert!(message(gio::IOErrorEnum::Cancelled).contains("cancelado"));
    assert!(message(gio::IOErrorEnum::PermissionDenied).contains("Android"));
}
