use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Keymaps {
    layouts: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unknown {
    Layout(String),
    Variant { variant: String, layout: String },
}

impl fmt::Display for Unknown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unknown::Layout(layout) => write!(formatter, "layout desconhecido: {layout}"),
            Unknown::Variant { variant, layout } => write!(formatter, "o layout {layout} não tem a variante {variant}"),
        }
    }
}

enum Block {
    Layouts,
    Variants,
    Other,
}

impl Keymaps {
    pub fn parse(rules: &str) -> Self {
        let mut keymaps = Self::default();
        let mut block = Block::Other;
        for line in rules.lines() {
            match line.trim() {
                "! layout" => block = Block::Layouts,
                "! variant" => block = Block::Variants,
                header if header.starts_with('!') => block = Block::Other,
                entry => keymaps.read(&block, entry),
            }
        }
        keymaps
    }

    pub fn check(&self, layout: &str, variant: &str) -> Result<(), Unknown> {
        if self.layouts.is_empty() {
            return Ok(());
        }
        let layouts: Vec<&str> = split(layout).collect();
        if let Some(missing) = layouts.iter().find(|name| !self.layouts.contains_key(**name)) {
            return Err(Unknown::Layout((*missing).to_owned()));
        }
        for (variant, layout) in split(variant).zip(layouts) {
            let known = self.layouts.get(layout).is_some_and(|variants| variants.contains(variant));
            if !variant.is_empty() && !known {
                return Err(Unknown::Variant { variant: variant.to_owned(), layout: layout.to_owned() });
            }
        }
        Ok(())
    }

    fn read(&mut self, block: &Block, entry: &str) {
        let mut words = entry.split_whitespace();
        let (Some(name), rest) = (words.next(), words.next()) else {
            return;
        };
        match block {
            Block::Layouts => {
                self.layouts.entry(name.to_owned()).or_default();
            }
            Block::Variants => {
                let Some(layout) = rest.and_then(|word| word.strip_suffix(':')) else {
                    return;
                };
                self.layouts.entry(layout.to_owned()).or_default().insert(name.to_owned());
            }
            Block::Other => {}
        }
    }
}

fn split(text: &str) -> impl Iterator<Item = &str> {
    text.split(',').map(str::trim).filter(|part| !part.is_empty() || text.contains(','))
}

#[cfg(test)]
mod tests {
    use super::{Keymaps, Unknown};

    const REGRAS: &str = "! model\n  pc105 Generic 105-key PC\n\n! layout\n  us              English (US)\n  br              Portuguese (Brazil)\n\n! variant\n  intl            us: English (US, intl., with dead keys)\n  abnt2           br: Portuguese (Brazil)\n  nodeadkeys      br: Portuguese (Brazil, no dead keys)\n\n! option\n  ctrl:nocaps     Caps Lock as Ctrl\n";

    #[test]
    fn aceita_layouts_e_variantes_conhecidos() {
        let keymaps = Keymaps::parse(REGRAS);
        assert_eq!(keymaps.check("br", "abnt2"), Ok(()));
        assert_eq!(keymaps.check("br,us", "abnt2,intl"), Ok(()));
        assert_eq!(keymaps.check("br,us", ",intl"), Ok(()));
        assert_eq!(keymaps.check("us", ""), Ok(()));
    }

    #[test]
    fn recusa_o_que_o_xkb_nao_conhece() {
        let keymaps = Keymaps::parse(REGRAS);
        assert_eq!(keymaps.check("bra", ""), Err(Unknown::Layout("bra".to_owned())));
        assert_eq!(
            keymaps.check("us", "abnt2"),
            Err(Unknown::Variant { variant: "abnt2".to_owned(), layout: "us".to_owned() })
        );
        assert!(keymaps.check("pc105", "").is_err());
    }

    #[test]
    fn sem_a_lista_do_xkb_nao_da_para_validar_e_tudo_passa() {
        assert_eq!(Keymaps::default().check("qualquer", "coisa"), Ok(()));
    }
}
