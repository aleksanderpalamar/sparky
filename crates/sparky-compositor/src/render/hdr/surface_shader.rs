use smithay::backend::renderer::gles::{
    GlesRenderer, GlesTexProgram, Uniform, UniformName, UniformType, UniformValue,
};

use super::glsl;
use crate::color_management::conversion::{Conversion, Decode};

const MIN_ALPHA: f64 = 0.0001;
const DECODES: [(&str, Decode); 4] = [
    ("DECODE_SRGB", Decode::Srgb),
    ("DECODE_GAMMA22", Decode::Gamma22),
    ("DECODE_LINEAR", Decode::Linear),
    ("DECODE_PQ", Decode::Pq),
];

const BODY: &str = r#"
uniform int transfer;
uniform mat3 primaries;
uniform float scale;

vec3 decode(vec3 color) {
    if (transfer == DECODE_PQ) {
        return pq_decode(color) / PEAK;
    }
    if (transfer == DECODE_GAMMA22) {
        return sign(color) * pow(abs(color), vec3(GAMMA22));
    }
    if (transfer == DECODE_LINEAR) {
        return color;
    }
    return srgb_decode(color);
}

void main() {
    vec4 color = texture2D(tex, v_coords);
#if defined(NO_ALPHA)
    color.a = 1.0;
#endif
    vec3 straight = color.rgb / max(color.a, MIN_ALPHA);
    vec3 light = primaries * (decode(straight) * scale);
    gl_FragColor = vec4(srgb_encode(light) * color.a, color.a) * alpha;
}
"#;

struct Cached(Option<GlesTexProgram>);

pub fn program(renderer: &mut GlesRenderer) -> Option<GlesTexProgram> {
    if let Some(Cached(program)) = renderer.egl_context().user_data().get::<Cached>() {
        return program.clone();
    }
    let compiled = match renderer.compile_custom_texture_shader(shader(), &names()) {
        Ok(program) => Some(program),
        Err(error) => {
            tracing::warn!("conversão de cor das janelas indisponível: {error}");
            None
        }
    };
    let cached = compiled.clone();
    renderer
        .egl_context()
        .user_data()
        .insert_if_missing(move || Cached(cached));
    compiled
}

pub fn uniforms(conversion: &Conversion) -> Vec<Uniform<'static>> {
    let matrix = conversion.primaries;
    let flat: [f32; 9] = std::array::from_fn(|index| matrix[index % 3][index / 3] as f32);
    vec![
        Uniform::new("transfer", code(conversion.decode)),
        Uniform::new(
            "primaries",
            UniformValue::Matrix3x3 {
                matrices: vec![flat],
                transpose: false,
            },
        ),
        Uniform::new("scale", conversion.scale as f32),
    ]
}

pub fn shader() -> String {
    let decodes = DECODES
        .map(|(name, decode)| format!("#define {name} {}", code(decode)))
        .join("\n");
    let defines = format!(
        "{}\n{}\n{decodes}",
        glsl::curve_defines(),
        glsl::define("MIN_ALPHA", MIN_ALPHA)
    );
    glsl::inject(
        &format!("{}{}{BODY}", glsl::PRELUDE, glsl::CURVES),
        &defines,
    )
}

fn names() -> [UniformName<'static>; 3] {
    [
        UniformName::new("transfer", UniformType::_1i),
        UniformName::new("primaries", UniformType::Matrix3x3),
        UniformName::new("scale", UniformType::_1f),
    ]
}

const fn code(decode: Decode) -> i32 {
    match decode {
        Decode::Srgb => 0,
        Decode::Gamma22 => 1,
        Decode::Linear => 2,
        Decode::Pq => 3,
    }
}

#[cfg(test)]
mod tests {
    use smithay::backend::renderer::gles::UniformValue;

    use super::{shader, uniforms};
    use crate::color_management::conversion::conversion;
    use crate::color_management::description::{
        ImageDescription, NamedPrimaries, TransferFunction,
    };

    #[test]
    fn shader_declara_as_curvas_e_os_parametros() {
        let fonte = shader();
        assert!(fonte.contains("//_DEFINES_"));
        assert_eq!(fonte.matches("#define DECODE_PQ 3").count(), 2);
        assert_eq!(fonte.matches("#define GAMMA22 2.2000000000").count(), 2);
        for uniform in [
            "uniform int transfer;",
            "uniform mat3 primaries;",
            "uniform float scale;",
        ] {
            assert!(fonte.contains(uniform), "{uniform}");
        }
        assert!(fonte.contains("vec3 pq_decode(vec3 signal)"));
        assert!(
            fonte.contains("gl_FragColor = vec4(srgb_encode(light) * color.a, color.a) * alpha;")
        );
    }

    #[test]
    fn matriz_vai_por_colunas_sem_transpor() -> Result<(), &'static str> {
        let description =
            ImageDescription::standard(TransferFunction::St2084Pq, NamedPrimaries::Bt2020);
        let conversion = conversion(&description).ok_or("PQ precisa de conversão")?;
        let valores = uniforms(&conversion);
        let nomes: Vec<&str> = valores
            .iter()
            .map(|uniform| uniform.name.as_ref())
            .collect();
        assert_eq!(nomes, ["transfer", "primaries", "scale"]);
        assert_eq!(valores[0].value, UniformValue::_1i(3));
        let UniformValue::Matrix3x3 {
            matrices,
            transpose,
        } = &valores[1].value
        else {
            return Err("primárias precisam ser uma mat3");
        };
        assert!(!transpose);
        let matriz = conversion.primaries;
        assert_eq!(matrices[0][1], matriz[1][0] as f32);
        assert_eq!(matrices[0][3], matriz[0][1] as f32);
        Ok(())
    }
}
