use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use md5::{Digest, Md5};

const LITERAL: &[u8] = b"!$&'()*+,-./:=@_~";

pub fn file_uri(path: &Path) -> String {
    path.as_os_str()
        .as_bytes()
        .iter()
        .fold(String::from("file://"), |mut uri, byte| {
            match byte.is_ascii_alphanumeric() || LITERAL.contains(byte) {
                true => uri.push(char::from(*byte)),
                false => uri.push_str(&format!("%{byte:02X}")),
            }
            uri
        })
}

pub fn cache_name(uri: &str) -> String {
    let digest = Md5::digest(uri.as_bytes());
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("{hex}.png")
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    use super::{cache_name, file_uri};

    #[test]
    fn nome_no_cache_segue_o_exemplo_da_especificacao() {
        assert_eq!(
            cache_name("file:///home/jens/photos/me.png"),
            "c6ee772d9e49320e97ec29a7eb5b1697.png"
        );
    }

    #[test]
    fn uri_escapa_espacos_e_acentos_como_o_glib() {
        assert_eq!(
            file_uri(Path::new("/home/ana/Fotos/praia de São Paulo.png")),
            "file:///home/ana/Fotos/praia%20de%20S%C3%A3o%20Paulo.png"
        );
        assert_eq!(
            file_uri(Path::new("/tmp/a#b?c%d[1]~x.png")),
            "file:///tmp/a%23b%3Fc%25d%5B1%5D~x.png"
        );
    }

    #[test]
    fn uri_preserva_bytes_que_nao_sao_utf8() {
        let path = Path::new(OsStr::from_bytes(b"/tmp/\xff.png"));
        assert_eq!(file_uri(path), "file:///tmp/%FF.png");
    }
}
