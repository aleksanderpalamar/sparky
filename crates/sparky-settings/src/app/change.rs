use sparky_config::{Editor, parse};

use crate::app::state::{Settings, Source, Target};
use crate::domain::field::Field;
use crate::domain::value::{Value, shown, store};
use crate::ports::CompositorError;

impl Settings {
    pub fn set(&mut self, field: Field, value: Value) {
        let label = format!("{}: {}", field.label(), shown(&value));
        self.edit(&label, |editor| store(field, &value, editor));
    }

    pub fn release(&mut self) {
        let Some((target, value)) = self.sliding.take() else {
            return;
        };
        match target {
            Target::Field(field) => self.set(field, Value::Number(value)),
            Target::System(id) => self.choose(id, value.to_string()),
            Target::Autostart(_) => {}
        }
    }

    pub fn set_autostart(&mut self, entries: Vec<String>) {
        self.edit(&crate::i18n::text("section-autostart-label"), |editor| {
            editor.set_list("autostart", &entries)
        });
    }

    pub(crate) fn edit(&mut self, what: &str, change: impl FnOnce(&mut Editor)) {
        let Source::Editable(editor) = &self.source else {
            return self.fail(crate::i18n::text("config-broken"));
        };
        let mut draft = editor.clone();
        change(&mut draft);
        let contents = draft.contents();
        let config = match parse(&contents) {
            Ok(config) => config,
            Err(error) => {
                return self.fail(crate::i18n::with(
                    "config-would-be-invalid",
                    "reason",
                    &error.to_string(),
                ));
            }
        };
        if let Err(error) = self.services.store.save(&contents) {
            return self.fail(crate::i18n::with(
                "save-failed",
                "reason",
                &error.to_string(),
            ));
        }
        self.source = Source::Editable(draft);
        self.config = config;
        self.announce(what);
    }

    fn announce(&mut self, what: &str) {
        match self.services.compositor.reload() {
            Ok(()) => self.inform(crate::i18n::with("saved-applied", "value", what)),
            Err(CompositorError::Unreachable(_)) => {
                self.inform(crate::i18n::with("saved-next-start", "value", what))
            }
            Err(refused) => self.fail(refused),
        }
    }
}
