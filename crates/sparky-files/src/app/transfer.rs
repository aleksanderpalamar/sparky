use iced::Task;

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::clip::Clip;
use crate::domain::destination;
use crate::domain::entry::Kind;
use crate::i18n;

impl Files {
    pub fn begin_destination(&mut self, clip: Clip) {
        if clip.paths().is_empty() {
            return;
        }
        self.input = self.location.display().to_string();
        self.mode = Mode::Destination(clip);
    }

    pub fn finish_destination(&mut self, clip: Clip, input: &str) -> Task<Message> {
        let Some(target) = destination::resolve(input, &self.location, &self.home) else {
            self.fail(i18n::text("need-destination"));
            return Task::none();
        };
        match self.services.disk.properties(&target) {
            Ok(details) if details.kind == Kind::Directory => {}
            Ok(_) => {
                self.fail(i18n::text("destination-not-folder"));
                return Task::none();
            }
            Err(error) => {
                self.fail(format!("Destino: {error}"));
                return Task::none();
            }
        }
        self.transfer(clip, target)
    }
}
