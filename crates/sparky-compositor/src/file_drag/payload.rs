pub const URI_LIST: &str = "text/uri-list";
pub const GNOME: &str = "x-special/gnome-copied-files";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload {
    uris: Vec<String>,
}

impl Payload {
    pub fn new(uris: Vec<String>) -> Option<Self> {
        let valid = !uris.is_empty() && uris.iter().all(|uri| acceptable(uri));
        valid.then_some(Self { uris })
    }

    pub fn mime_types() -> Vec<String> {
        vec![URI_LIST.to_owned(), GNOME.to_owned()]
    }

    pub fn content(&self, mime_type: &str) -> Option<String> {
        match mime_type {
            URI_LIST => Some(self.uris.iter().map(|uri| format!("{uri}\r\n")).collect()),
            GNOME => Some(
                std::iter::once("copy")
                    .chain(self.uris.iter().map(String::as_str))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            _ => None,
        }
    }
}

fn acceptable(uri: &str) -> bool {
    uri.starts_with("file:///") && !uri.contains(['\r', '\n', '\0'])
}

#[cfg(test)]
mod tests {
    use super::{GNOME, Payload, URI_LIST};

    fn uris(lista: &[&str]) -> Vec<String> {
        lista.iter().map(|uri| (*uri).to_owned()).collect()
    }

    #[test]
    fn oferece_lista_de_uris_e_formato_do_gnome_como_copia() {
        let Some(payload) = Payload::new(uris(&["file:///a/x.txt", "file:///a/b%20c"])) else {
            panic!("os uris deveriam valer")
        };
        assert_eq!(
            payload.content(URI_LIST).as_deref(),
            Some("file:///a/x.txt\r\nfile:///a/b%20c\r\n")
        );
        assert_eq!(
            payload.content(GNOME).as_deref(),
            Some("copy\nfile:///a/x.txt\nfile:///a/b%20c")
        );
        assert_eq!(payload.content("image/png"), None);
        assert_eq!(Payload::mime_types(), [URI_LIST, GNOME]);
    }

    #[test]
    fn so_aceita_arquivos_locais_sem_quebra_de_linha() {
        assert_eq!(Payload::new(Vec::new()), None);
        assert_eq!(Payload::new(uris(&["https://x"])), None);
        assert_eq!(Payload::new(uris(&["file:///a\nfile:///b"])), None);
        assert_eq!(Payload::new(uris(&["file://host/a"])), None);
    }
}
