use crate::app::state::{Files, Mode};
use crate::domain::clip::Clip;
use crate::domain::destination;
use crate::domain::entry::Kind;
use crate::domain::wording::{agreed, items};
use crate::operations;

impl Files {
    pub fn begin_destination(&mut self, clip: Clip) {
        if clip.paths().is_empty() {
            return;
        }
        self.input = self.location.display().to_string();
        self.mode = Mode::Destination(clip);
    }

    pub fn finish_destination(&mut self, clip: &Clip, input: &str) {
        let Some(target) = destination::resolve(input, &self.location, &self.home) else {
            self.fail("Informe uma pasta de destino");
            return;
        };
        match self.services.disk.properties(&target) {
            Ok(details) if details.kind == Kind::Directory => {}
            Ok(_) => {
                self.fail("O destino não é uma pasta");
                return;
            }
            Err(error) => {
                self.fail(format!("Destino: {error}"));
                return;
            }
        }
        let result = operations::paste(clip, &target, self.services.disk.as_ref());
        self.refresh();
        match result {
            Ok(names) => self.inform(format!(
                "{} {} para {}",
                items(names.len()),
                agreed(clip.verb(), names.len()),
                target.display()
            )),
            Err(error) => self.fail(error),
        }
    }
}
