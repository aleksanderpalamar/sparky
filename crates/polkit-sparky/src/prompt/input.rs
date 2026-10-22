use std::io::{BufRead, BufReader, Read};

use tokio::sync::oneshot;
use zeroize::Zeroizing;

use crate::domain::Request;

const MAX_BYTES: usize = 65536;

pub fn read() -> Result<(Request, oneshot::Receiver<()>), String> {
    let mut reader = BufReader::new(std::io::stdin());
    let request = decode(&mut reader)?;
    let (sender, cancellation) = oneshot::channel();
    std::thread::Builder::new()
        .name("polkit-parent".to_owned())
        .spawn(move || {
            let _ = reader.read(&mut [0u8; 1]);
            let _ = sender.send(());
        })
        .map_err(|error| error.to_string())?;
    Ok((request, cancellation))
}

pub fn decode(reader: &mut impl BufRead) -> Result<Request, String> {
    let mut bytes = Zeroizing::new(Vec::new());
    let mut limited = Read::by_ref(reader).take((MAX_BYTES + 1) as u64);
    limited
        .read_until(b'\n', &mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_BYTES || bytes.last() != Some(&b'\n') {
        return Err("Pedido privado incompleto ou grande demais".to_owned());
    }
    let request: Request = serde_json::from_slice(&bytes).map_err(|_| "Pedido privado inválido")?;
    request.validate()?;
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::{MAX_BYTES, decode};

    #[test]
    fn canal_privado_recusa_conteudo_invalido_e_nao_expoe_cookie() {
        assert!(
            decode(&mut "cookie-super-secreto\n".as_bytes())
                .is_err_and(|error| !error.contains("super-secreto"))
        );
        assert!(decode(&mut "{}".as_bytes()).is_err());
        assert!(decode(&mut vec![b'x'; MAX_BYTES + 1].as_slice()).is_err());
    }
}
