pub mod choice;
pub mod decode;
pub mod refresh;

#[cfg(test)]
mod wallpaper_tests;

use std::cell::OnceCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;

use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::memory::MemoryRenderBuffer;
use smithay::reexports::calloop::LoopHandle;
use smithay::reexports::calloop::channel::{self, Event, Sender};
use smithay::utils::Transform;

use crate::state::Sparky;
use crate::wallpaper::choice::Choice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wanted {
    pub output: String,
    pub path: PathBuf,
    pub size: (u32, u32),
}

pub struct Decoded {
    pub wanted: Wanted,
    pub pixels: Result<Vec<u8>, String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Settled {
    Painted,
    Stale,
    Failed(String),
}

#[derive(Default)]
pub struct Wallpaper {
    choice: Choice,
    painted: HashMap<String, (Wanted, MemoryRenderBuffer)>,
    pending: Vec<Wanted>,
    sender: Option<Sender<Decoded>>,
    fallback: OnceCell<Option<PathBuf>>,
}

impl Wallpaper {
    pub fn new(handle: &LoopHandle<'static, Sparky>) -> Self {
        let (sender, receiver) = channel::channel();
        let inserted = handle.insert_source(receiver, |event, _, state| {
            if let Event::Msg(decoded) = event {
                state.receive_wallpaper(decoded);
            }
        });
        match inserted {
            Ok(_) => Self { sender: Some(sender), ..Self::default() },
            Err(error) => {
                tracing::warn!("sem papel de parede: {error}");
                Self::default()
            }
        }
    }

    pub fn plan(&mut self, choice: Choice, outputs: &[(String, (u32, u32))]) -> Vec<Wanted> {
        self.choice = choice;
        self.painted.retain(|output, _| outputs.iter().any(|(name, _)| name == output));
        let Choice::File(path) = &self.choice else {
            self.painted.clear();
            return Vec::new();
        };
        let fresh: Vec<Wanted> = outputs
            .iter()
            .map(|(output, size)| Wanted { output: output.clone(), path: path.clone(), size: *size })
            .filter(|wanted| self.painted.get(&wanted.output).is_none_or(|(painted, _)| painted != wanted))
            .filter(|wanted| !self.pending.contains(wanted))
            .collect();
        self.pending.extend(fresh.iter().cloned());
        fresh
    }

    pub fn settle(&mut self, decoded: Decoded) -> Settled {
        let wanted = decoded.wanted;
        self.pending.retain(|pending| *pending != wanted);
        if self.choice != Choice::File(wanted.path.clone()) {
            return Settled::Stale;
        }
        match decoded.pixels {
            Ok(pixels) => {
                let (width, height) = wanted.size;
                let buffer = MemoryRenderBuffer::from_slice(&pixels, Fourcc::Abgr8888, (width as i32, height as i32), 1, Transform::Normal, None);
                self.painted.insert(wanted.output.clone(), (wanted, buffer));
                Settled::Painted
            }
            Err(error) => {
                self.painted.remove(&wanted.output);
                Settled::Failed(error)
            }
        }
    }

    pub fn fallback(&self, find: impl FnOnce() -> Option<PathBuf>) -> Option<PathBuf> {
        self.fallback.get_or_init(find).clone()
    }

    pub fn buffer(&self, output: &str) -> Option<&MemoryRenderBuffer> {
        self.painted.get(output).map(|(_, buffer)| buffer)
    }

    pub fn decode(&self, wanted: Wanted) {
        let Some(sender) = self.sender.clone() else {
            return;
        };
        let spawned = thread::Builder::new().name("sparky-wallpaper".into()).spawn(move || {
            let pixels = decode::decode(&wanted.path, wanted.size);
            if sender.send(Decoded { wanted, pixels }).is_err() {
                tracing::debug!("papel de parede pronto depois do fim do sparky");
            }
        });
        if let Err(error) = spawned {
            tracing::warn!("não deu para carregar o papel de parede: {error}");
        }
    }
}
