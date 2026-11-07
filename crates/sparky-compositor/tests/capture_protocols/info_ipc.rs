use sparky_ipc::{Reply, Request, SessionKind};

use super::client::Client;
use super::fixture::Error;

#[test]
fn o_compositor_informa_versao_display_sessao_e_cores() -> Result<(), Error> {
    let client = Client::connect("ipc")?;
    let Reply::Info { info } = client.ipc()?.request(&Request::Info)? else {
        return Err("resposta inesperada ao pedido info".into());
    };
    assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    assert!(!info.display.is_empty());
    assert_eq!(info.session, SessionKind::Nested);
    assert_eq!(
        (info.accent.len(), info.accent.chars().next()),
        (7, Some('#'))
    );
    assert!(info.swatches.len() >= 6);
    assert!(info.swatches.iter().all(|swatch| swatch.len() == 7));
    Ok(())
}
