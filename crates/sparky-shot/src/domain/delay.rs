use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Delay {
    #[default]
    Immediate,
    Three,
    Five,
    Ten,
}

impl Delay {
    pub fn next(self) -> Self {
        match self {
            Delay::Immediate => Delay::Three,
            Delay::Three => Delay::Five,
            Delay::Five => Delay::Ten,
            Delay::Ten => Delay::Immediate,
        }
    }

    pub fn duration(self) -> Duration {
        Duration::from_secs(match self {
            Delay::Immediate => 0,
            Delay::Three => 3,
            Delay::Five => 5,
            Delay::Ten => 10,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Delay::Immediate => "sem atraso",
            Delay::Three => "3 segundos",
            Delay::Five => "5 segundos",
            Delay::Ten => "10 segundos",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Delay;
    use std::time::Duration;

    #[test]
    fn tab_percorre_os_atrasos_e_volta_ao_comeco() {
        let mut delay = Delay::default();
        let mut seen = Vec::new();
        for _ in 0..5 {
            seen.push(delay.duration());
            delay = delay.next();
        }
        assert_eq!(
            seen,
            [0, 3, 5, 10, 0].map(Duration::from_secs).to_vec()
        );
    }

    #[test]
    fn cada_atraso_tem_um_rotulo() {
        assert_eq!(Delay::Immediate.label(), "sem atraso");
        assert_eq!(Delay::Ten.label(), "10 segundos");
    }
}
