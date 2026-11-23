pub mod payload;
pub mod source;

use smithay::input::dnd::DnDGrab;
use smithay::input::pointer::Focus;
use smithay::utils::SERIAL_COUNTER;
use sparky_ipc::Reply;

use crate::file_drag::payload::Payload;
use crate::file_drag::source::FileSource;
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
        let grab = DnDGrab::new_pointer(
            &self.display_handle,
            start,
            FileSource::new(payload),
            self.seat.clone(),
        );
        pointer.set_grab(self, grab, SERIAL_COUNTER.next_serial(), Focus::Keep);
        Reply::Done
    }
}

fn refusal(message: &str) -> Reply {
    Reply::Error {
        message: message.to_owned(),
    }
}
