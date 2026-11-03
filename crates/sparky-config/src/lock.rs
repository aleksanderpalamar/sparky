use std::time::Duration;

use serde::{Deserialize, Serialize};

const IDLE_MINUTES: u32 = 10;
const MINUTE: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Lock {
    pub idle_minutes: u32,
}

impl Lock {
    pub fn idle_limit(&self) -> Option<Duration> {
        match self.idle_minutes {
            0 => None,
            minutes => Some(Duration::from_secs(u64::from(minutes) * MINUTE)),
        }
    }
}

impl Default for Lock {
    fn default() -> Self {
        Self {
            idle_minutes: IDLE_MINUTES,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Lock;
    use crate::load::parse;
    use crate::schema::Config;

    #[test]
    fn por_padrao_bloqueia_depois_de_dez_minutos_parado() {
        assert_eq!(Config::default().lock, Lock { idle_minutes: 10 });
        assert_eq!(
            Config::default().lock.idle_limit(),
            Some(Duration::from_secs(600))
        );
    }

    #[test]
    fn le_o_tempo_e_zero_desliga() {
        let Ok(config) = parse("[lock]\nidle_minutes = 5\n") else {
            panic!("a seção lock deveria ser aceita");
        };
        assert_eq!(config.lock.idle_limit(), Some(Duration::from_secs(300)));
        let Ok(config) = parse("[lock]\nidle_minutes = 0\n") else {
            panic!("zero deveria ser aceito");
        };
        assert_eq!(config.lock.idle_limit(), None);
    }

    #[test]
    fn tempo_negativo_e_recusado() {
        assert!(parse("[lock]\nidle_minutes = -1\n").is_err());
    }
}
