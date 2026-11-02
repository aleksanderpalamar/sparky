use super::glsl;
use crate::color_management::{curves, gamut};

const SDR_WHITE: f64 = 203.0;
const BT709_TO_BT2020: [[f64; 3]; 3] = [
    [0.627_404_0, 0.329_282_0, 0.043_313_6],
    [0.069_097_0, 0.919_540_0, 0.011_361_2],
    [0.016_391_6, 0.088_013_2, 0.895_595_0],
];

const BODY: &str = r#"
uniform float peak;

void main() {
    vec4 color = texture2D(tex, v_coords);
    vec3 wide = BT2020 * srgb_decode(color.rgb);
    gl_FragColor = vec4(pq_encode(min(wide * SDR_WHITE, vec3(peak))), 1.0) * alpha;
}
"#;

pub fn to_bt2020(rgb: [f64; 3]) -> [f64; 3] {
    gamut::apply(&BT709_TO_BT2020, rgb)
}

pub fn encode(srgb: [f64; 3]) -> [f64; 3] {
    to_bt2020(srgb.map(curves::srgb_to_linear)).map(|channel| curves::pq(channel * SDR_WHITE))
}

pub fn shader() -> String {
    let constants = format!(
        "{}\n{}",
        glsl::curve_defines(),
        glsl::define("SDR_WHITE", SDR_WHITE)
    );
    let columns: Vec<String> = (0..3)
        .flat_map(|column| (0..3).map(move |row| format!("{:.7}", BT709_TO_BT2020[row][column])))
        .collect();
    let matrix = format!("#define BT2020 mat3({})", columns.join(", "));
    glsl::inject(
        &format!("{}{}{BODY}", glsl::PRELUDE, glsl::CURVES),
        &format!("{constants}\n{matrix}"),
    )
}

#[cfg(test)]
mod tests {
    use super::{encode, shader, to_bt2020};

    fn perto(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn valor_acima_do_branco_passa_do_branco_do_sdr() {
        let branco = encode([1.0, 1.0, 1.0])[0];
        let acima = encode([2.0, 2.0, 2.0])[0];
        assert!(acima > branco, "{acima} <= {branco}");
        assert!(encode([-0.5, -0.5, -0.5]).iter().all(|canal| *canal < 1e-5));
    }

    #[test]
    fn branco_continua_branco_e_cores_perdem_o_exagero() {
        let branco = to_bt2020([1.0, 1.0, 1.0]);
        assert!(branco.iter().all(|canal| (canal - 1.0).abs() < 1e-5));
        let vermelho = encode([1.0, 0.0, 0.0]);
        for (canal, esperado) in vermelho.into_iter().zip([0.532_546, 0.327_023, 0.220_07]) {
            assert!(perto(canal, esperado), "{vermelho:?}");
        }
        assert!(
            encode([1.0, 1.0, 1.0])
                .iter()
                .all(|canal| perto(*canal, 0.580_689))
        );
    }

    #[test]
    fn o_shader_usa_as_mesmas_constantes() {
        let fonte = shader();
        assert!(fonte.contains("//_DEFINES_"));
        assert_eq!(fonte.matches("#define M2 78.8437500000").count(), 2);
        assert_eq!(
            fonte
                .matches("#define BT2020 mat3(0.6274040, 0.0690970, 0.0163916,")
                .count(),
            2
        );
        assert_eq!(fonte.matches("#define SDR_WHITE 203.0000000000").count(), 2);
        assert!(!fonte.contains("uniform float sdr_white;"));
        assert!(!fonte.contains("clamp(color.rgb, 0.0, 1.0)"));
        assert!(fonte.contains("sign(color)"));
        assert!(fonte.contains("uniform float peak;"));
        assert!(fonte.contains("pq_encode(min(wide * SDR_WHITE, vec3(peak)))"));
    }
}
