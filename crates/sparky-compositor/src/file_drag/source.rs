use std::io::Write;
use std::os::fd::OwnedFd;
use std::sync::atomic::{AtomicBool, Ordering};

use smithay::input::dnd::{DndAction, Source, SourceMetadata};
use smithay::utils::IsAlive;

use crate::file_drag::payload::Payload;

pub struct FileSource {
    payload: Payload,
    alive: AtomicBool,
}

impl FileSource {
    pub fn new(payload: Payload) -> Self {
        Self {
            payload,
            alive: AtomicBool::new(true),
        }
    }

    fn end(&self) {
        self.alive.store(false, Ordering::Release);
    }
}

impl IsAlive for FileSource {
    fn alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }
}

impl Source for FileSource {
    fn metadata(&self) -> Option<SourceMetadata> {
        Some(SourceMetadata {
            mime_types: Payload::mime_types(),
            dnd_actions: std::iter::once(DndAction::Copy).collect(),
        })
    }

    fn choose_action(&self, _action: DndAction) {}

    fn send(&self, mime_type: &str, fd: OwnedFd) {
        let Some(content) = self.payload.content(mime_type) else {
            return;
        };
        std::thread::spawn(move || {
            let mut target = std::fs::File::from(fd);
            if let Err(error) = target.write_all(content.as_bytes()) {
                eprintln!("sparky: arraste de arquivos não entregue: {error}");
            }
        });
    }

    fn drop_performed(&self) {}

    fn cancel(&self) {
        self.end();
    }

    fn finished(&self) {
        self.end();
    }
}
