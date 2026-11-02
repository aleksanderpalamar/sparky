use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::{Id, RenderElement};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexProgram, GlesTexture};
use smithay::backend::renderer::utils::CommitCounter;
use smithay::backend::renderer::{Bind, Color32F, Offscreen};
use smithay::output::Output;
use smithay::utils::{Physical, Size, Transform};

use crate::render::hdr::element::PqElement;

pub struct Composite {
    id: Id,
    commit: CommitCounter,
    texture: GlesTexture,
    damage: OutputDamageTracker,
    size: Size<i32, Physical>,
    scale: f64,
    drawn: bool,
}

impl Composite {
    pub fn new(renderer: &mut GlesRenderer, output: &Output) -> Result<Self, String> {
        let (size, scale) = target(output).ok_or("monitor sem modo de vídeo")?;
        let texture = renderer
            .create_buffer(Fourcc::Abgr16161616f, (size.w, size.h).into())
            .map_err(|error| error.to_string())?;
        Ok(Self {
            id: Id::new(),
            commit: CommitCounter::default(),
            texture,
            damage: OutputDamageTracker::new(size, scale, Transform::Normal),
            size,
            scale,
            drawn: false,
        })
    }

    pub fn fits(&self, output: &Output) -> bool {
        target(output) == Some((self.size, self.scale))
    }

    pub fn draw<E: RenderElement<GlesRenderer>>(
        &mut self,
        renderer: &mut GlesRenderer,
        elements: &[E],
        clear: impl Into<Color32F>,
    ) -> Result<(), String> {
        let age = usize::from(self.drawn);
        let mut framebuffer = renderer
            .bind(&mut self.texture)
            .map_err(|error| error.to_string())?;
        let result = self
            .damage
            .render_output(renderer, &mut framebuffer, age, elements, clear)
            .map_err(|error| format!("{error:?}"))?;
        if result.damage.is_some_and(|damage| !damage.is_empty()) {
            self.commit.increment();
        }
        self.drawn = true;
        Ok(())
    }

    pub fn element(&self, program: &GlesTexProgram) -> PqElement {
        PqElement {
            id: self.id.clone(),
            commit: self.commit,
            texture: self.texture.clone(),
            size: self.size,
            program: program.clone(),
        }
    }
}

fn target(output: &Output) -> Option<(Size<i32, Physical>, f64)> {
    let mode = output.current_mode()?;
    let size = output.current_transform().transform_size(mode.size);
    Some((size, output.current_scale().fractional_scale()))
}
