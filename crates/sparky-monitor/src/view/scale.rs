use crate::domain::units::rate;

const MIN_RATE: f32 = 1000.0;
const DIVISIONS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Percent,
    Rate,
}

pub fn ceiling(scale: Scale, peak: f32) -> f32 {
    match scale {
        Scale::Percent => 100.0,
        Scale::Rate => nice(peak.max(MIN_RATE)),
    }
}

pub fn labels(scale: Scale, ceiling: f32) -> Vec<String> {
    (0..=DIVISIONS)
        .rev()
        .map(|step| ceiling * step as f32 / DIVISIONS as f32)
        .map(|value| match scale {
            Scale::Percent => format!("{value:.0}%"),
            Scale::Rate => rate(f64::from(value)),
        })
        .collect()
}

fn nice(value: f32) -> f32 {
    let magnitude = 10f32.powf(value.log10().floor());
    let step = match value / magnitude {
        normalized if normalized <= 1.0 => 1.0,
        normalized if normalized <= 2.0 => 2.0,
        normalized if normalized <= 5.0 => 5.0,
        _ => 10.0,
    };
    step * magnitude
}

#[cfg(test)]
mod tests {
    use super::{Scale, ceiling, labels};

    #[test]
    fn taxa_arredonda_o_pico_para_cima_em_passos_de_1_2_e_5() {
        assert_eq!(ceiling(Scale::Rate, 0.0), 1000.0);
        assert_eq!(ceiling(Scale::Rate, 1500.0), 2000.0);
        assert_eq!(ceiling(Scale::Rate, 413_700.0), 500_000.0);
        assert_eq!(ceiling(Scale::Rate, 6_000_000.0), 10_000_000.0);
        assert_eq!(ceiling(Scale::Percent, 3.0), 100.0);
    }

    #[test]
    fn rotulos_vao_do_teto_ao_zero() {
        assert_eq!(
            labels(Scale::Percent, 100.0),
            ["100%", "75%", "50%", "25%", "0%"]
        );
        assert_eq!(
            labels(Scale::Rate, 2_000_000.0),
            [
                "2,00 MB/s",
                "1,50 MB/s",
                "1,00 MB/s",
                "500,00 kB/s",
                "0 B/s"
            ]
        );
    }
}
