use std::ffi::OsString;
use std::path::PathBuf;

const SUFFIXES: [&str; 13] = [
    ".tar.gz", ".tgz", ".tar.bz2", ".tbz2", ".tar.xz", ".txz", ".tar.zst", ".tzst", ".tar.lz4",
    ".tar", ".zip", ".7z", ".rar",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extraction {
    Done(PathBuf),
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placement {
    Single(OsString),
    Folder,
}

pub fn extractable(name: &str) -> Option<&str> {
    let lower = name.to_lowercase();
    SUFFIXES
        .iter()
        .find(|suffix| lower.len() > suffix.len() && lower.ends_with(*suffix))
        .and_then(|suffix| name.get(..name.len() - suffix.len()))
        .filter(|stem| !stem.is_empty())
}

pub fn placement(mut tops: Vec<OsString>) -> Placement {
    match (tops.pop(), tops.is_empty()) {
        (Some(only), true) => Placement::Single(only),
        _ => Placement::Folder,
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{Placement, extractable, placement};

    #[test]
    fn reconhece_zip_7z_e_variantes_de_tar_sem_caixa() {
        assert_eq!(extractable("fotos.zip"), Some("fotos"));
        assert_eq!(extractable("Backup.TAR.GZ"), Some("Backup"));
        assert_eq!(extractable("dados.tar.zst"), Some("dados"));
        assert_eq!(extractable("app.7z"), Some("app"));
        assert_eq!(extractable("fonte.tgz"), Some("fonte"));
        assert_eq!(extractable("notas.txt"), None);
        assert_eq!(extractable(".zip"), None);
        assert_eq!(extractable("gz"), None);
    }

    #[test]
    fn um_unico_item_no_topo_sai_direto_e_varios_ganham_pasta() {
        assert_eq!(
            placement(vec![OsString::from("projeto")]),
            Placement::Single(OsString::from("projeto"))
        );
        assert_eq!(
            placement(vec![OsString::from("a"), OsString::from("b")]),
            Placement::Folder
        );
        assert_eq!(placement(Vec::new()), Placement::Folder);
    }
}
