use std::path::PathBuf;

use chrono::NaiveDateTime;

pub const FOLDER: &str = "Capturas de tela";
const KEY: &str = "XDG_PICTURES_DIR";

pub fn pictures_dir(user_dirs: Option<&str>, home: &str) -> PathBuf {
    user_dirs
        .and_then(|contents| configured(contents, home))
        .unwrap_or_else(|| PathBuf::from(home).join("Pictures"))
}

pub fn file_name(moment: &NaiveDateTime) -> String {
    format!("captura-{}.png", moment.format("%Y-%m-%d_%H-%M-%S"))
}

fn configured(contents: &str, home: &str) -> Option<PathBuf> {
    let value = contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == KEY)
        .map(|(_, value)| value.trim().trim_matches('"'))?;
    let expanded = value.replace("$HOME", home);
    (!expanded.is_empty()).then(|| PathBuf::from(expanded))
}

#[cfg(test)]
mod tests {
    use super::{file_name, pictures_dir};
    use chrono::NaiveDate;
    use std::path::PathBuf;

    #[test]
    fn le_a_pasta_de_imagens_do_usuario() {
        let user_dirs = "XDG_DESKTOP_DIR=\"$HOME/Área de trabalho\"\nXDG_PICTURES_DIR=\"$HOME/Imagens\"\n";
        assert_eq!(pictures_dir(Some(user_dirs), "/home/palamar"), PathBuf::from("/home/palamar/Imagens"));
    }

    #[test]
    fn caminho_absoluto_fica_como_esta() {
        assert_eq!(pictures_dir(Some("XDG_PICTURES_DIR=\"/dados/fotos\"\n"), "/h"), PathBuf::from("/dados/fotos"));
    }

    #[test]
    fn sem_configuracao_vale_pictures_na_pasta_pessoal() {
        assert_eq!(pictures_dir(None, "/home/palamar"), PathBuf::from("/home/palamar/Pictures"));
        assert_eq!(pictures_dir(Some("# nada\n"), "/home/palamar"), PathBuf::from("/home/palamar/Pictures"));
    }

    #[test]
    fn o_nome_leva_data_e_hora_e_ordena_bem() {
        let Some(momento) = NaiveDate::from_ymd_opt(2026, 10, 7).and_then(|d| d.and_hms_opt(9, 5, 3)) else {
            panic!("momento inválido");
        };
        assert_eq!(file_name(&momento), "captura-2026-10-07_09-05-03.png");
    }
}
