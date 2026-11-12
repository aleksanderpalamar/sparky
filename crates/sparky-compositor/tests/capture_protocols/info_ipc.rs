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

fn pedir(client: &Client, uris: &[&str]) -> Result<Reply, Error> {
    let request = Request::Drag {
        uris: uris.iter().map(|uri| (*uri).to_owned()).collect(),
    };
    Ok(client.ipc()?.request(&request)?)
}

#[test]
fn arrastar_sem_botao_pressionado_ou_sem_arquivo_e_recusado() -> Result<(), Error> {
    let client = Client::connect("ipc")?;
    let sem_botao = pedir(&client, &["file:///tmp/a.txt"])?;
    assert_eq!(
        sem_botao,
        Reply::Error {
            message: "nenhum botão pressionado sobre uma janela".to_owned()
        }
    );
    let remoto = pedir(&client, &["https://exemplo.com"])?;
    assert!(matches!(remoto, Reply::Error { .. }));
    Ok(())
}
