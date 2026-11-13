use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::domain::clip_payload::path_of;

pub const SELECT: &str = "--select";
pub const PROPERTIES: &str = "--properties";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Start {
    Folder(Option<PathBuf>),
    Select(Vec<PathBuf>),
    Properties(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    Folders,
    Items,
    Properties,
}

pub fn parse(args: &[String]) -> Start {
    match args {
        [flag, items @ ..] if flag == SELECT && !items.is_empty() => {
            Start::Select(items.iter().map(PathBuf::from).collect())
        }
        [flag, item, ..] if flag == PROPERTIES => Start::Properties(PathBuf::from(item)),
        [folder, ..] if !folder.starts_with("--") => Start::Folder(Some(PathBuf::from(folder))),
        _ => Start::Folder(None),
    }
}

pub fn windows(request: Request, uris: &[String]) -> Vec<Vec<String>> {
    let paths: Vec<PathBuf> = uris.iter().filter_map(|uri| path_of(uri)).collect();
    match request {
        Request::Folders => paths.iter().map(|path| vec![shown(path)]).collect(),
        Request::Properties => paths
            .iter()
            .map(|path| vec![PROPERTIES.to_owned(), shown(path)])
            .collect(),
        Request::Items => by_parent(&paths)
            .into_values()
            .map(|items| {
                std::iter::once(SELECT.to_owned())
                    .chain(items.iter().map(|path| shown(path)))
                    .collect()
            })
            .collect(),
    }
}

fn by_parent(paths: &[PathBuf]) -> BTreeMap<PathBuf, Vec<&PathBuf>> {
    let mut groups: BTreeMap<PathBuf, Vec<&PathBuf>> = BTreeMap::new();
    for path in paths {
        let parent = path.parent().map(PathBuf::from).unwrap_or_default();
        groups.entry(parent).or_default().push(path);
    }
    groups
}

fn shown(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Request, Start, parse, windows};

    fn textos(lista: &[&str]) -> Vec<String> {
        lista.iter().map(|texto| (*texto).to_owned()).collect()
    }

    #[test]
    fn argumentos_escolhem_pasta_selecao_ou_propriedades() {
        assert_eq!(parse(&[]), Start::Folder(None));
        assert_eq!(
            parse(&textos(&["/tmp"])),
            Start::Folder(Some(PathBuf::from("/tmp")))
        );
        assert_eq!(
            parse(&textos(&["--select", "/a/x", "/a/y"])),
            Start::Select(vec![PathBuf::from("/a/x"), PathBuf::from("/a/y")])
        );
        assert_eq!(
            parse(&textos(&["--properties", "/a/x"])),
            Start::Properties(PathBuf::from("/a/x"))
        );
        assert_eq!(parse(&textos(&["--select"])), Start::Folder(None));
        assert_eq!(parse(&textos(&["--outro"])), Start::Folder(None));
    }

    #[test]
    fn mostrar_itens_abre_uma_janela_por_pasta_com_todos_selecionados() {
        let uris = textos(&[
            "file:///home/ana/Fotos/a%20b.jpg",
            "https://exemplo.com/x",
            "file:///home/ana/nota.txt",
            "file:///home/ana/Fotos/c.jpg",
        ]);
        assert_eq!(
            windows(Request::Items, &uris),
            vec![
                textos(&["--select", "/home/ana/nota.txt"]),
                textos(&[
                    "--select",
                    "/home/ana/Fotos/a b.jpg",
                    "/home/ana/Fotos/c.jpg"
                ]),
            ]
        );
    }

    #[test]
    fn pastas_e_propriedades_abrem_uma_janela_por_item() {
        let uris = textos(&["file:///srv", "file:///tmp/x"]);
        assert_eq!(
            windows(Request::Folders, &uris),
            vec![textos(&["/srv"]), textos(&["/tmp/x"])]
        );
        assert_eq!(
            windows(Request::Properties, &uris[1..]),
            vec![textos(&["--properties", "/tmp/x"])]
        );
    }
}
