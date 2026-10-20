use std::path::Path;

const SOURCES: [&str; 2] = ["/etc/vconsole.conf", "/etc/default/keyboard"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardLayout {
    pub layout: String,
    pub variant: String,
    pub model: String,
    pub options: String,
}

impl Default for KeyboardLayout {
    fn default() -> Self {
        Self {
            layout: "us".to_owned(),
            variant: String::new(),
            model: String::new(),
            options: String::new(),
        }
    }
}

impl KeyboardLayout {
    pub fn from_system(read: impl Fn(&Path) -> Option<String>) -> Self {
        SOURCES
            .iter()
            .filter_map(|source| read(Path::new(source)))
            .map(|contents| Self::parse(&contents))
            .find(|layout| !layout.layout.is_empty())
            .unwrap_or_default()
    }

    pub fn parse(contents: &str) -> Self {
        Self {
            layout: value(contents, "XKBLAYOUT"),
            variant: value(contents, "XKBVARIANT"),
            model: value(contents, "XKBMODEL"),
            options: value(contents, "XKBOPTIONS"),
        }
    }
}

fn value(contents: &str, key: &str) -> String {
    contents
        .lines()
        .filter_map(|line| line.trim().split_once('='))
        .find(|(name, _)| name.trim() == key)
        .map(|(_, value)| value.trim().trim_matches(['"', '\'']).to_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::KeyboardLayout;
    use std::path::Path;

    #[test]
    fn le_o_layout_do_vconsole() {
        let layout = KeyboardLayout::parse("KEYMAP=br-abnt2\nXKBLAYOUT=br\nXKBVARIANT=abnt2\n");
        assert_eq!(layout.layout, "br");
        assert_eq!(layout.variant, "abnt2");
    }

    #[test]
    fn tira_as_aspas_do_debian() {
        let layout = KeyboardLayout::parse("XKBLAYOUT=\"br\"\nXKBOPTIONS=\"ctrl:nocaps\"\n");
        assert_eq!(layout.layout, "br");
        assert_eq!(layout.options, "ctrl:nocaps");
    }

    #[test]
    fn prefere_o_vconsole_e_cai_no_debian() {
        let ler = |path: &Path| match path.to_str() {
            Some("/etc/vconsole.conf") => Some("KEYMAP=us\n".to_owned()),
            Some("/etc/default/keyboard") => Some("XKBLAYOUT=\"pt\"\n".to_owned()),
            _ => None,
        };
        assert_eq!(KeyboardLayout::from_system(ler).layout, "pt");
    }

    #[test]
    fn sem_arquivo_fica_us() {
        assert_eq!(KeyboardLayout::from_system(|_| None).layout, "us");
    }
}
