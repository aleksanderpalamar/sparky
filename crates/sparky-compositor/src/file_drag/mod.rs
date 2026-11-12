pub mod payload;

use std::io::Write;
use std::os::fd::OwnedFd;

use smithay::reexports::wayland_server::protocol::wl_data_device_manager::DndAction;
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::selection::data_device::{SourceMetadata, start_dnd};
use sparky_ipc::Reply;

use crate::file_drag::payload::Payload;
use crate::state::Sparky;

impl Sparky {
    pub fn start_file_drag(&mut self, uris: Vec<String>) -> Reply {
        let Some(payload) = Payload::new(uris) else {
            return refusal("somente arquivos locais podem ser arrastados");
        };
        let Some(pointer) = self.seat.get_pointer() else {
            return refusal("sem ponteiro");
        };
        let Some(start) = pointer
            .grab_start_data()
            .filter(|start| start.focus.is_some())
        else {
            return refusal("nenhum botão pressionado sobre uma janela");
        };
        let metadata = SourceMetadata {
            mime_types: Payload::mime_types(),
            dnd_action: DndAction::Copy,
        };
        self.file_drag = Some(payload);
        let seat = self.seat.clone();
        let display = self.display_handle.clone();
        start_dnd(
            &display,
            &seat,
            self,
            SERIAL_COUNTER.next_serial(),
            Some(start),
            None,
            metadata,
        );
        Reply::Done
    }

    pub fn send_file_drag(&mut self, mime_type: &str, fd: OwnedFd) {
        let Some(content) = self
            .file_drag
            .as_ref()
            .and_then(|payload| payload.content(mime_type))
        else {
            return;
        };
        std::thread::spawn(move || {
            let mut target = std::fs::File::from(fd);
            let _ = target.write_all(content.as_bytes());
        });
    }

    pub fn end_file_drag(&mut self) {
        self.file_drag = None;
    }
}

fn refusal(message: &str) -> Reply {
    Reply::Error {
        message: message.to_owned(),
    }
}
