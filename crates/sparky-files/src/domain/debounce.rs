use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Debounce {
    window: Duration,
    since: Option<Instant>,
}

impl Debounce {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            since: None,
        }
    }

    pub fn note(&mut self, now: Instant) {
        self.since.get_or_insert(now);
    }

    pub fn fire(&mut self, now: Instant) -> bool {
        match self.since {
            Some(since) if now.saturating_duration_since(since) >= self.window => {
                self.since = None;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::Debounce;

    const JANELA: Duration = Duration::from_millis(400);

    #[test]
    fn sem_eventos_nao_dispara() {
        let mut debounce = Debounce::new(JANELA);
        assert!(!debounce.fire(Instant::now() + JANELA * 3));
    }

    #[test]
    fn varios_eventos_na_janela_disparam_uma_vez() {
        let inicio = Instant::now();
        let mut debounce = Debounce::new(JANELA);
        debounce.note(inicio);
        debounce.note(inicio + Duration::from_millis(100));
        assert!(!debounce.fire(inicio + Duration::from_millis(399)));
        assert!(debounce.fire(inicio + JANELA));
        assert!(!debounce.fire(inicio + JANELA * 2));
    }

    #[test]
    fn escrita_continua_nao_adia_a_atualizacao() {
        let inicio = Instant::now();
        let mut debounce = Debounce::new(JANELA);
        let disparos = (0..20)
            .map(|passo| inicio + Duration::from_millis(passo * 100))
            .filter(|agora| {
                debounce.note(*agora);
                debounce.fire(*agora)
            })
            .count();
        assert_eq!(disparos, 4);
    }
}
