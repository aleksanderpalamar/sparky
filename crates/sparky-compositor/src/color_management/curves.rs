pub const PEAK_NITS: f64 = 10000.0;
pub const M1: f64 = 2610.0 / 16384.0;
pub const M2: f64 = 2523.0 / 4096.0 * 128.0;
pub const C1: f64 = 3424.0 / 4096.0;
pub const C2: f64 = 2413.0 / 4096.0 * 32.0;
pub const C3: f64 = 2392.0 / 4096.0 * 32.0;
const SRGB_KNEE: f64 = 0.04045;
const LINEAR_KNEE: f64 = 0.003_130_8;
pub const GAMMA22: f64 = 2.2;

pub fn srgb_to_linear(value: f64) -> f64 {
    let magnitude = value.abs();
    let linear = match magnitude <= SRGB_KNEE {
        true => magnitude / 12.92,
        false => ((magnitude + 0.055) / 1.055).powf(2.4),
    };
    linear.copysign(value)
}

pub fn linear_to_srgb(value: f64) -> f64 {
    let magnitude = value.abs();
    let encoded = match magnitude <= LINEAR_KNEE {
        true => magnitude * 12.92,
        false => 1.055 * magnitude.powf(1.0 / 2.4) - 0.055,
    };
    encoded.copysign(value)
}

pub fn gamma22_to_linear(value: f64) -> f64 {
    value.abs().powf(GAMMA22).copysign(value)
}

pub fn pq(nits: f64) -> f64 {
    let y = (nits / PEAK_NITS).clamp(0.0, 1.0).powf(M1);
    ((C1 + C2 * y) / (1.0 + C3 * y)).powf(M2)
}

pub fn pq_to_nits(signal: f64) -> f64 {
    let power = signal.clamp(0.0, 1.0).powf(1.0 / M2);
    let ratio = (power - C1).max(0.0) / (C2 - C3 * power);
    ratio.powf(1.0 / M1) * PEAK_NITS
}

#[cfg(test)]
mod tests {
    use super::{gamma22_to_linear, linear_to_srgb, pq, pq_to_nits, srgb_to_linear};

    fn perto(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn curva_pq_segue_a_st2084() {
        for (nits, esperado) in [
            (0.0, 0.000_001),
            (100.0, 0.508_078),
            (203.0, 0.580_689),
            (1000.0, 0.751_827),
            (10000.0, 1.0),
        ] {
            assert!(perto(pq(nits), esperado), "{nits} nits: {}", pq(nits));
        }
        assert!(perto(pq(20000.0), 1.0));
    }

    #[test]
    fn pq_decodificado_volta_aos_nits() {
        for nits in [0.0, 0.5, 100.0, 203.0, 1000.0, 4000.0, 10000.0] {
            let volta = pq_to_nits(pq(nits));
            assert!(
                (volta - nits).abs() < 1e-6 * nits.max(1.0),
                "{nits}: {volta}"
            );
        }
        assert!(perto(pq_to_nits(1.5), 10000.0));
        assert!(perto(pq_to_nits(-0.2), 0.0));
    }

    #[test]
    fn srgb_vira_luz_linear() {
        assert!(perto(srgb_to_linear(0.5), 0.214_041));
        assert!(perto(srgb_to_linear(0.02), 0.02 / 12.92));
        assert!(perto(srgb_to_linear(1.0), 1.0));
    }

    #[test]
    fn srgb_estendido_continua_a_curva_e_preserva_o_sinal() {
        let acima = ((2.0_f64 + 0.055) / 1.055).powf(2.4);
        assert!(perto(srgb_to_linear(2.0), acima));
        assert!(perto(srgb_to_linear(-0.5), -srgb_to_linear(0.5)));
        assert!(perto(srgb_to_linear(-0.02), -0.02 / 12.92));
    }

    #[test]
    fn luz_linear_volta_ao_srgb_estendido() {
        for valor in [-2.0, -0.5, -0.01, 0.0, 0.002, 0.04, 0.5, 1.0, 3.0] {
            assert!(
                perto(linear_to_srgb(srgb_to_linear(valor)), valor),
                "{valor}"
            );
        }
    }

    #[test]
    fn gama_2_2_e_potencia_pura_com_sinal() {
        assert!(perto(gamma22_to_linear(0.5), 0.5_f64.powf(2.2)));
        assert!(perto(gamma22_to_linear(-0.5), -(0.5_f64.powf(2.2))));
        assert!(perto(gamma22_to_linear(1.0), 1.0));
    }
}
