use smithay::backend::renderer::gles::{
    GlesError, GlesRenderer, GlesTexProgram, UniformName, UniformType,
};

const DOWN: &str = include_str!("down.frag");
const UP: &str = include_str!("up.frag");
const CLIP: &str = include_str!("clip.frag");

#[derive(Debug, Clone)]
pub struct Shaders {
    pub down: GlesTexProgram,
    pub up: GlesTexProgram,
    pub clip: GlesTexProgram,
}

impl Shaders {
    pub fn get(renderer: &mut GlesRenderer) -> Result<Self, GlesError> {
        if let Some(shaders) = renderer.egl_context().user_data().get::<Shaders>() {
            return Ok(shaders.clone());
        }
        let shaders = Self {
            down: compile(renderer, DOWN)?,
            up: compile(renderer, UP)?,
            clip: renderer.compile_custom_texture_shader(
                CLIP,
                &[
                    UniformName::new("rect", UniformType::_4f),
                    UniformName::new("size", UniformType::_2f),
                    UniformName::new("radius", UniformType::_1f),
                    UniformName::new("grain", UniformType::_1f),
                ],
            )?,
        };
        renderer
            .egl_context()
            .user_data()
            .insert_if_missing(|| shaders.clone());
        Ok(shaders)
    }
}

fn compile(renderer: &mut GlesRenderer, source: &str) -> Result<GlesTexProgram, GlesError> {
    renderer.compile_custom_texture_shader(
        source,
        &[
            UniformName::new("half_pixel", UniformType::_2f),
            UniformName::new("offset", UniformType::_1f),
        ],
    )
}
