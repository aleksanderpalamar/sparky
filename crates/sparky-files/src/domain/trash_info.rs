use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};

use chrono::NaiveDateTime;

pub fn info(original: &Path, moment: &NaiveDateTime) -> String {
    format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        encode(original.as_os_str().as_bytes()),
        moment.format("%Y-%m-%dT%H:%M:%S")
    )
}

fn encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                char::from(*byte).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

pub fn original_in(record: &str, topdir: Option<&Path>) -> io::Result<PathBuf> {
    let path = recorded(record)?;
    match (path.is_absolute(), topdir) {
        (true, _) => Ok(path),
        (false, Some(topdir))
            if path
                .components()
                .all(|part| matches!(part, Component::Normal(_))) =>
        {
            Ok(topdir.join(path))
        }
        _ => Err(invalid_path()),
    }
}

pub fn original(record: &str) -> io::Result<PathBuf> {
    original_in(record, None)
}

fn recorded(record: &str) -> io::Result<PathBuf> {
    let mut section = false;
    for line in record.lines() {
        if line.starts_with('[') {
            section = line == "[Trash Info]";
            continue;
        }
        let Some(encoded) = section.then(|| line.strip_prefix("Path=")).flatten() else {
            continue;
        };
        let bytes = decode(encoded.as_bytes())?;
        if bytes.contains(&0) {
            return Err(invalid_path());
        }
        return Ok(PathBuf::from(OsString::from_vec(bytes)));
    }
    Err(invalid_path())
}

fn decode(encoded: &[u8]) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(encoded.len());
    let mut index = 0;
    while index < encoded.len() {
        if encoded[index] != b'%' {
            bytes.push(encoded[index]);
            index += 1;
            continue;
        }
        let pair = encoded.get(index + 1..index + 3).ok_or_else(invalid_path)?;
        let code = std::str::from_utf8(pair).map_err(|_| invalid_path())?;
        let byte = u8::from_str_radix(code, 16).map_err(|_| invalid_path())?;
        bytes.push(byte);
        index += 3;
    }
    Ok(bytes)
}

fn invalid_path() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        crate::i18n::text("trash-invalid-path"),
    )
}

#[cfg(test)]
mod tests {
    use super::{info, original, original_in};
    use chrono::NaiveDate;
    use std::path::Path;

    #[test]
    fn grava_o_caminho_codificado_e_a_data_da_exclusao() {
        let Some(momento) =
            NaiveDate::from_ymd_opt(2026, 10, 20).and_then(|d| d.and_hms_opt(16, 5, 9))
        else {
            panic!("momento inválido");
        };
        assert_eq!(
            info(
                Path::new("/home/palamar/Área de trabalho/a+b.txt"),
                &momento
            ),
            "[Trash Info]\nPath=/home/palamar/%C3%81rea%20de%20trabalho/a%2Bb.txt\nDeletionDate=2026-10-20T16:05:09\n"
        );
    }

    #[test]
    fn recupera_caminho_original_com_acentos_e_espacos() {
        let record = "[Trash Info]\nPath=/home/p/Imagens/%C3%81lbum%20novo.png\nDeletionDate=2026-10-28T12:00:00\n";
        assert_eq!(
            original(record).ok().as_deref(),
            Some(Path::new("/home/p/Imagens/Álbum novo.png"))
        );
    }

    #[test]
    fn rejeita_caminhos_relativos_e_percentual_invalido() {
        assert!(original("[Trash Info]\nPath=relative/file.txt\n").is_err());
        assert!(original("[Trash Info]\nPath=/home/p/%GG.txt\n").is_err());
        assert!(original("Path=/home/p/arquivo.txt\n").is_err());
    }

    #[test]
    fn caminho_relativo_so_vale_dentro_da_raiz_do_disco() {
        let registro = "[Trash Info]\nPath=Fotos/praia%20nova.png\n";
        assert_eq!(
            original_in(registro, Some(Path::new("/run/media/ana/USB"))).ok(),
            Some(Path::new("/run/media/ana/USB/Fotos/praia nova.png").to_path_buf())
        );
        assert!(original_in("[Trash Info]\nPath=../segredo\n", Some(Path::new("/m"))).is_err());
        assert!(original_in(registro, None).is_err());
    }
}
