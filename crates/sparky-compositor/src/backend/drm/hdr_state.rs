use crate::backend::drm::hdr_props::HdrProperties;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HdrState {
    #[default]
    Off,
    Pending,
    On {
        properties: HdrProperties,
        blob: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Enable,
    Disable {
        properties: HdrProperties,
        blob: u64,
    },
    Cancel,
    Keep,
}

impl HdrState {
    pub const fn requested(wanted: bool) -> Self {
        match wanted {
            true => Self::Pending,
            false => Self::Off,
        }
    }

    pub const fn step(self, wanted: bool) -> Step {
        match (wanted, self) {
            (true, Self::Off) => Step::Enable,
            (false, Self::Pending) => Step::Cancel,
            (false, Self::On { properties, blob }) => Step::Disable { properties, blob },
            _ => Step::Keep,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HdrState, Step};
    use crate::backend::drm::hdr_props::exemplo;

    fn ligado() -> HdrState {
        HdrState::On {
            properties: exemplo(),
            blob: 7,
        }
    }

    #[test]
    fn hdr_pedido_ao_ligar_a_tela_espera_o_primeiro_quadro() {
        assert_eq!(HdrState::requested(true), HdrState::Pending);
        assert_eq!(HdrState::requested(false), HdrState::Off);
    }

    #[test]
    fn hdr_pendente_nao_liga_antes_do_primeiro_quadro() {
        assert_eq!(HdrState::Pending.step(true), Step::Keep);
        assert_eq!(HdrState::Pending.step(false), Step::Cancel);
    }

    #[test]
    fn hdr_desligado_liga_quando_a_configuracao_pede() {
        assert_eq!(HdrState::Off.step(true), Step::Enable);
        assert_eq!(HdrState::Off.step(false), Step::Keep);
    }

    #[test]
    fn hdr_ligado_so_desliga_quando_a_configuracao_tira() {
        let disable = Step::Disable {
            properties: exemplo(),
            blob: 7,
        };
        assert_eq!(ligado().step(false), disable);
        assert_eq!(ligado().step(true), Step::Keep);
    }
}
