use super::test_client::Client;
use super::test_server::{DEVICE, Error, formats};

#[test]
fn gpu_anuncia_protocolo_com_feedback() -> Result<(), Error> {
    let client = Client::connect(Some(DEVICE), 4)?;
    assert!(
        client.advertised >= 4,
        "DMA-BUF sem feedback: versão {}",
        client.advertised
    );
    Ok(())
}

#[test]
fn feedback_identifica_gpu_e_todos_os_formatos() -> Result<(), Error> {
    let mut client = Client::connect(Some(DEVICE), 4)?;
    assert!(client.advertised >= 4);
    client.feedback()?;
    let events = &client.events;
    assert!(events.done);
    assert_eq!(events.error, None);
    assert_eq!(events.device, Some(DEVICE));
    assert_eq!(events.targets, vec![DEVICE]);
    let table = events
        .table
        .chunks_exact(16)
        .map(format_entry)
        .collect::<Result<Vec<_>, _>>()?;
    let expected: Vec<_> = formats()
        .iter()
        .map(|format| (format.code as u32, u64::from(format.modifier)))
        .collect();
    assert_eq!(table, expected);
    assert_eq!(events.indices, vec![0, 1]);
    Ok(())
}

#[test]
fn cliente_v3_ainda_recebe_os_formatos_da_gpu() -> Result<(), Error> {
    let client = Client::connect(Some(DEVICE), 3)?;
    let expected: Vec<_> = formats()
        .iter()
        .map(|format| (format.code as u32, u64::from(format.modifier)))
        .collect();
    assert_eq!(client.events.formats, expected);
    Ok(())
}

#[test]
fn sem_gpu_preserva_compatibilidade_v3() -> Result<(), Error> {
    let client = Client::connect(None, 3)?;
    assert_eq!(client.advertised, 3);
    assert_eq!(client.events.formats.len(), formats().iter().count());
    Ok(())
}

fn format_entry(entry: &[u8]) -> Result<(u32, u64), std::array::TryFromSliceError> {
    let code = u32::from_ne_bytes(entry[..4].try_into()?);
    let modifier = u64::from_ne_bytes(entry[8..16].try_into()?);
    Ok((code, modifier))
}
