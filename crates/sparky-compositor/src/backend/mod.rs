pub mod drm;
pub mod host_cursor;
pub mod nested;

use std::mem;

use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::{ImportDma, ImportMemWl};

use crate::backend::drm::Drm;
use crate::backend::nested::Nested;
use crate::state::Sparky;

pub enum Backend {
    Starting,
    Nested(Box<Nested>),
    Drm(Box<Drm>),
}

impl Sparky {
    pub fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        let renderer = match &mut self.backend {
            Backend::Nested(nested) => Some(nested.graphics.renderer()),
            Backend::Drm(drm) => Some(&mut drm.gpu.renderer),
            Backend::Starting => None,
        };
        renderer.is_some_and(|renderer| renderer.import_dmabuf(dmabuf, None).is_ok())
    }

    pub fn switch_terminal(&mut self, terminal: i32) {
        if let Backend::Drm(drm) = &mut self.backend {
            drm.switch_terminal(terminal);
        }
    }

    pub fn render_pending(&mut self) {
        self.with_drm(drm::render::render_all);
    }

    pub fn with_drm<T>(&mut self, change: impl FnOnce(&mut Sparky, &mut Drm) -> T) -> Option<T> {
        match mem::replace(&mut self.backend, Backend::Starting) {
            Backend::Drm(mut drm) => {
                let result = change(self, &mut drm);
                self.backend = Backend::Drm(drm);
                Some(result)
            }
            other => {
                self.backend = other;
                None
            }
        }
    }

    pub fn with_nested<T>(&mut self, change: impl FnOnce(&mut Sparky, &mut Nested) -> T) -> Option<T> {
        match mem::replace(&mut self.backend, Backend::Starting) {
            Backend::Nested(mut nested) => {
                let result = change(self, &mut nested);
                self.backend = Backend::Nested(nested);
                Some(result)
            }
            other => {
                self.backend = other;
                None
            }
        }
    }

    pub fn with_renderer<T>(&mut self, use_renderer: impl FnOnce(&mut Sparky, &mut GlesRenderer) -> T) -> Option<T> {
        match mem::replace(&mut self.backend, Backend::Starting) {
            Backend::Nested(mut nested) => {
                let result = use_renderer(self, nested.graphics.renderer());
                self.backend = Backend::Nested(nested);
                Some(result)
            }
            Backend::Drm(mut drm) => {
                let result = use_renderer(self, &mut drm.gpu.renderer);
                self.backend = Backend::Drm(drm);
                Some(result)
            }
            Backend::Starting => None,
        }
    }

    pub fn publish_formats(&mut self, renderer: &GlesRenderer) {
        self.protocols.shm.update_formats(renderer.shm_formats());
        let formats = renderer.dmabuf_formats();
        let global = self.protocols.dmabuf.create_global::<Sparky>(&self.display_handle, formats);
        self.dmabuf_global = Some(global);
    }
}
