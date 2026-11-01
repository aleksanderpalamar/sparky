const SDR_WHITE: f64 = 203.0;
const PEAK_NITS: f64 = 10000.0;
const M1: f64 = 2610.0 / 16384.0;
const M2: f64 = 2523.0 / 4096.0 * 128.0;
const C1: f64 = 3424.0 / 4096.0;
const C2: f64 = 2413.0 / 4096.0 * 32.0;
const C3: f64 = 2392.0 / 4096.0 * 32.0;
const BT709_TO_BT2020: [[f64; 3]; 3] = [
    [0.627_404_0, 0.329_282_0, 0.043_313_6],
    [0.069_097_0, 0.919_540_0, 0.011_361_2],
    [0.016_391_6, 0.088_013_2, 0.895_595_0],
];

const SHADER: &str = r#"#version 100

//_DEFINES_

#if defined(EXTERNAL)
#extension GL_OES_EGL_image_external : require
#endif

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

#if defined(EXTERNAL)
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif

uniform float alpha;
varying vec2 v_coords;

#if defined(DEBUG_FLAGS)
uniform float tint;
#endif

vec3 linear(vec3 color) {
    vec3 low = color / 12.92;
    vec3 high = pow((color + 0.055) / 1.055, vec3(2.4));
    return mix(low, high, step(vec3(0.04045), color));
}

vec3 perceptual(vec3 nits) {
    vec3 y = pow(clamp(nits / PEAK, 0.0, 1.0), vec3(M1));
    return pow((C1 + C2 * y) / (1.0 + C3 * y), vec3(M2));
}

void main() {
    vec4 color = texture2D(tex, v_coords);
    vec3 wide = BT2020 * linear(clamp(color.rgb, 0.0, 1.0));
    gl_FragColor = vec4(perceptual(wide * SDR_WHITE), 1.0) * alpha;
}
"#;

pub fn srgb_to_linear(value: f64) -> f64 {
    match value <= 0.04045 {
        true => value / 12.92,
        false => ((value + 0.055) / 1.055).powf(2.4),
    }
}

pub fn pq(nits: f64) -> f64 {
    let y = (nits / PEAK_NITS).clamp(0.0, 1.0).powf(M1);
    ((C1 + C2 * y) / (1.0 + C3 * y)).powf(M2)
}

pub fn to_bt2020(rgb: [f64; 3]) -> [f64; 3] {
    BT709_TO_BT2020.map(|row| {
        row.iter()
            .zip(rgb)
            .map(|(weight, channel)| weight * channel)
            .sum()
    })
}

pub fn encode(srgb: [f64; 3]) -> [f64; 3] {
    to_bt2020(srgb.map(srgb_to_linear)).map(|channel| pq(channel * SDR_WHITE))
}

pub fn shader() -> String {
    let constants = [
        ("PEAK", PEAK_NITS),
        ("SDR_WHITE", SDR_WHITE),
        ("M1", M1),
        ("M2", M2),
        ("C1", C1),
        ("C2", C2),
        ("C3", C3),
    ]
    .map(|(name, value)| format!("#define {name} {value:.10}"))
    .join("\n");
    let columns: Vec<String> = (0..3)
        .flat_map(|column| (0..3).map(move |row| format!("{:.7}", BT709_TO_BT2020[row][column])))
        .collect();
    let matrix = format!("#define BT2020 mat3({})", columns.join(", "));
    SHADER
        .replacen(
            "precision highp float;",
            &format!("precision highp float;\n{constants}\n{matrix}"),
            1,
        )
        .replacen(
            "precision mediump float;\n#endif",
            &format!("precision mediump float;\n{constants}\n{matrix}\n#endif"),
            1,
        )
}

#[cfg(test)]
mod tests {
    use super::{encode, pq, shader, srgb_to_linear, to_bt2020};

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
    fn srgb_vira_luz_linear() {
        assert!(perto(srgb_to_linear(0.5), 0.214_041));
        assert!(perto(srgb_to_linear(0.02), 0.02 / 12.92));
        assert!(perto(srgb_to_linear(1.0), 1.0));
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
    }
}
