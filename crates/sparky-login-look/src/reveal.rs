#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reveal {
    Hidden,
    Shown,
}

impl Reveal {
    pub fn toggled(self) -> Self {
        match self {
            Reveal::Hidden => Reveal::Shown,
            Reveal::Shown => Reveal::Hidden,
        }
    }

    pub fn secure(self) -> bool {
        self == Reveal::Hidden
    }
}

#[cfg(test)]
mod tests {
    use super::Reveal;

    #[test]
    fn o_olho_alterna_entre_esconder_e_mostrar() {
        assert_eq!(Reveal::Hidden.toggled(), Reveal::Shown);
        assert_eq!(Reveal::Shown.toggled(), Reveal::Hidden);
        assert!(Reveal::Hidden.secure());
        assert!(!Reveal::Shown.secure());
    }
}
