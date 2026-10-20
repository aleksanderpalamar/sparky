use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => char::from(*byte).to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::info;
    use chrono::NaiveDate;
    use std::path::Path;

    #[test]
    fn grava_o_caminho_codificado_e_a_data_da_exclusao() {
        let Some(momento) = NaiveDate::from_ymd_opt(2026, 10, 20).and_then(|d| d.and_hms_opt(16, 5, 9)) else {
            panic!("momento inválido");
        };
        assert_eq!(
            info(Path::new("/home/palamar/Área de trabalho/a+b.txt"), &momento),
            "[Trash Info]\nPath=/home/palamar/%C3%81rea%20de%20trabalho/a%2Bb.txt\nDeletionDate=2026-10-20T16:05:09\n"
        );
    }
}
