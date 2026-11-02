use crate::color_management::curves::{C1, C2, C3, GAMMA22, M1, M2, PEAK_NITS};

pub const PRELUDE: &str = r#"#version 100

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
"#;

pub const CURVES: &str = r#"
vec3 srgb_decode(vec3 color) {
    vec3 magnitude = abs(color);
    vec3 low = magnitude / 12.92;
    vec3 high = pow((magnitude + 0.055) / 1.055, vec3(2.4));
    return sign(color) * mix(low, high, step(vec3(0.04045), magnitude));
}

vec3 srgb_encode(vec3 light) {
    vec3 magnitude = abs(light);
    vec3 low = magnitude * 12.92;
    vec3 high = 1.055 * pow(magnitude, vec3(1.0 / 2.4)) - 0.055;
    return sign(light) * mix(low, high, step(vec3(0.0031308), magnitude));
}

vec3 pq_encode(vec3 nits) {
    vec3 y = pow(clamp(nits / PEAK, 0.0, 1.0), vec3(M1));
    return pow((C1 + C2 * y) / (1.0 + C3 * y), vec3(M2));
}

vec3 pq_decode(vec3 signal) {
    vec3 power = pow(clamp(signal, 0.0, 1.0), vec3(1.0 / M2));
    return pow(max(power - C1, 0.0) / (C2 - C3 * power), vec3(1.0 / M1)) * PEAK;
}
"#;

pub fn curve_defines() -> String {
    [
        ("PEAK", PEAK_NITS),
        ("M1", M1),
        ("M2", M2),
        ("C1", C1),
        ("C2", C2),
        ("C3", C3),
        ("GAMMA22", GAMMA22),
    ]
    .map(|(name, value)| define(name, value))
    .join("\n")
}

pub fn define(name: &str, value: f64) -> String {
    format!("#define {name} {value:.10}")
}

pub fn inject(source: &str, block: &str) -> String {
    source
        .replacen(
            "precision highp float;",
            &format!("precision highp float;\n{block}"),
            1,
        )
        .replacen(
            "precision mediump float;\n#endif",
            &format!("precision mediump float;\n{block}\n#endif"),
            1,
        )
}
