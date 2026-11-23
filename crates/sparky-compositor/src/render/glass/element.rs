use std::sync::Mutex;

use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::{Element, Id, Kind, RenderElement};
use smithay::backend::renderer::gles::{GlesError, GlesFrame, GlesRenderer, GlesTexture};
use smithay::backend::renderer::utils::{CommitCounter, DamageSet};
use smithay::backend::renderer::{FrameContext, Offscreen, Texture};
use smithay::utils::user_data::UserDataMap;
use smithay::utils::{Buffer, Logical, Physical, Point, Rectangle, Scale, Size, Transform};

use super::capture::copy_behind;
use super::clip::pieces;
use super::paint::paint;
use super::passes::blur;
use super::shaders::Shaders;
use super::state::GlassState;

pub const PASSES: usize = 3;
pub const OFFSET: f32 = 3.0;
pub const REACH: i32 = 40;

type Cached = Mutex<Option<GlesTexture>>;

#[derive(Debug)]
pub struct GlassElement {
    id: Id,
    commit: CommitCounter,
    geometry: Rectangle<f64, Logical>,
    margin: Point<f64, Logical>,
    src: Size<f64, Buffer>,
    region: Vec<Rectangle<i32, Logical>>,
    shaders: Shaders,
    radius: f32,
}

impl GlassElement {
    pub fn new(
        shaders: Shaders,
        state: &mut GlassState,
        region: Vec<Rectangle<i32, Logical>>,
        scale: f64,
        radius: i32,
    ) -> Option<Self> {
        let bounds = region.iter().copied().reduce(|a, b| a.merge(b))?;
        if bounds.is_empty() {
            return None;
        }
        let margin = Point::<f64, Physical>::new(REACH as f64, REACH as f64).to_logical(scale);
        let mut geometry = bounds.to_f64();
        geometry.loc -= margin;
        geometry.size += margin.to_size().upscale(2.0);
        let src = geometry.size.to_buffer(scale, Transform::Normal);
        let local = region
            .iter()
            .map(|rect| Rectangle::new(rect.loc - bounds.loc + margin.to_i32_round(), rect.size))
            .collect::<Vec<_>>();
        state.touch(&local, src);
        Some(Self {
            id: state.id.clone(),
            commit: state.commit,
            geometry,
            margin,
            src,
            region: local,
            shaders,
            radius: radius as f32,
        })
    }
}

impl Element for GlassElement {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit
    }

    fn src(&self) -> Rectangle<f64, Buffer> {
        Rectangle::from_size(self.src)
    }

    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.geometry.to_physical_precise_round(scale)
    }

    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        if self
            .commit
            .distance(commit)
            .is_some_and(|distance| distance == 0)
        {
            return DamageSet::default();
        }
        let inner = self.geometry.size.to_physical_precise_round(scale)
            - self
                .margin
                .to_size()
                .upscale(2.0)
                .to_physical_precise_round(scale);
        DamageSet::from_slice(&[Rectangle::new(
            self.margin.to_physical_precise_round(scale),
            inner,
        )])
    }

    fn kind(&self) -> Kind {
        Kind::default()
    }

    fn is_framebuffer_effect(&self) -> bool {
        true
    }
}

impl RenderElement<GlesRenderer> for GlassElement {
    fn capture_framebuffer(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        cache: &UserDataMap,
    ) -> Result<(), GlesError> {
        let size = self.src.to_i32_round();
        let slot = cache.get_or_insert_threadsafe(Cached::default);
        let mut cached = slot.lock().map_err(|_| GlesError::BlitError)?;
        if cached
            .as_ref()
            .is_some_and(|texture| texture.size() != size)
        {
            cached.take();
        }
        let mut guard = frame.renderer();
        let mut scratch: GlesTexture = guard.as_mut().create_buffer(Fourcc::Abgr8888, size)?;
        let texture = match cached.as_mut() {
            Some(texture) => texture,
            None => cached.insert(guard.as_mut().create_buffer(Fourcc::Abgr8888, size)?),
        };
        drop(guard);
        copy_behind(frame, src, dst, texture)?;
        let mut guard = frame.renderer();
        blur(
            guard.as_mut(),
            &self.shaders,
            &mut [texture, &mut scratch],
            OFFSET,
            PASSES,
        )
    }

    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        cache: Option<&UserDataMap>,
    ) -> Result<(), GlesError> {
        let Some(slot) = cache.and_then(|cache| cache.get::<Cached>()) else {
            return Ok(());
        };
        let cached = slot.lock().map_err(|_| GlesError::BlitError)?;
        let Some(texture) = cached.as_ref() else {
            return Ok(());
        };
        let visible = src.upscale(self.geometry.size / self.src).to_logical(
            1.0,
            Transform::Normal,
            &Size::default(),
        );
        let pieces = pieces(
            &self.region,
            visible,
            dst,
            damage,
            self.geometry.size,
            self.radius,
        );
        for piece in pieces {
            paint(
                frame,
                texture,
                &self.shaders,
                piece,
                src,
                dst,
                opaque_regions,
            )?;
        }
        Ok(())
    }
}
