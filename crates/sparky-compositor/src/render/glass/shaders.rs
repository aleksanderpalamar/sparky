use smithay::backend::renderer::gles::{
    GlesError, GlesRenderer, GlesTexProgram, UniformName, UniformType,
};

const DOWN: &str = include_str!("down.frag");
const UP: &str = include_str!("up.frag");

#[derive(Debug, Clone)]
pub struct Shaders {
    pub down: GlesTexProgram,
    pub up: GlesTexProgram,
}

impl Shaders {
    pub fn get(renderer: &mut GlesRenderer) -> Result<Self, GlesError> {
        if let Some(shaders) = renderer.egl_context().user_data().get::<Shaders>() {
            return Ok(shaders.clone());
        }
        let shaders = Self {
            down: compile(renderer, DOWN)?,
            up: compile(renderer, UP)?,
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
