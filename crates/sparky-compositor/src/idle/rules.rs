use std::time::Duration;

use crate::lock::rules::Phase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inhibition {
    Free,
    Inhibited,
}

pub fn due(idle: Duration, limit: Option<Duration>, inhibition: Inhibition, phase: Phase) -> bool {
    match (limit, inhibition, phase) {
        (Some(limit), Inhibition::Free, Phase::Open) => idle >= limit,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Inhibition, due};
    use crate::lock::rules::Phase;

    const DEZ: Duration = Duration::from_secs(600);

    #[test]
    fn bloqueia_quando_o_tempo_parado_chega_ao_limite() {
        assert!(!due(
            DEZ - Duration::from_secs(1),
            Some(DEZ),
            Inhibition::Free,
            Phase::Open
        ));
        assert!(due(DEZ, Some(DEZ), Inhibition::Free, Phase::Open));
    }

    #[test]
    fn sem_limite_nunca_bloqueia() {
        assert!(!due(DEZ * 10, None, Inhibition::Free, Phase::Open));
    }

    #[test]
    fn um_video_tocando_segura_o_bloqueio() {
        assert!(!due(DEZ * 2, Some(DEZ), Inhibition::Inhibited, Phase::Open));
    }

    #[test]
    fn com_a_tela_ja_bloqueada_nao_bloqueia_de_novo() {
        assert!(!due(DEZ * 2, Some(DEZ), Inhibition::Free, Phase::Locked));
    }
}
