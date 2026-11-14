#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagColor {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
}

pub const ALL: [TagColor; 7] = [
    TagColor::Red,
    TagColor::Orange,
    TagColor::Yellow,
    TagColor::Green,
    TagColor::Blue,
    TagColor::Purple,
    TagColor::Pink,
];

impl TagColor {
    pub fn key(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Orange => "orange",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::Blue => "blue",
            Self::Purple => "purple",
            Self::Pink => "pink",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        ALL.into_iter().find(|color| color.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Red => "tag-red",
            Self::Orange => "tag-orange",
            Self::Yellow => "tag-yellow",
            Self::Green => "tag-green",
            Self::Blue => "tag-blue",
            Self::Purple => "tag-purple",
            Self::Pink => "tag-pink",
        }
    }

    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Red => (0xe0, 0x1b, 0x24),
            Self::Orange => (0xff, 0x78, 0x00),
            Self::Yellow => (0xf6, 0xd3, 0x2d),
            Self::Green => (0x33, 0xd1, 0x7a),
            Self::Blue => (0x35, 0x84, 0xe4),
            Self::Purple => (0x91, 0x41, 0xac),
            Self::Pink => (0xe6, 0x61, 0x98),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ALL, TagColor};

    #[test]
    fn as_sete_cores_seguem_a_ordem_da_paleta() {
        let chaves: Vec<_> = ALL.iter().map(|cor| cor.key()).collect();
        assert_eq!(
            chaves,
            ["red", "orange", "yellow", "green", "blue", "purple", "pink"]
        );
    }

    #[test]
    fn a_chave_gravada_volta_para_a_mesma_cor_e_chave_desconhecida_e_ignorada() {
        assert!(
            ALL.iter()
                .all(|cor| TagColor::from_key(cor.key()) == Some(*cor))
        );
        assert_eq!(TagColor::from_key("cinza"), None);
    }
}
