use sparky_config::{Editor, parse};

use crate::app::state::{Settings, Source};
use crate::domain::field::Field;
use crate::domain::value::{Value, read, shown, store};
use crate::ports::CompositorError;

impl Settings {
    pub fn set(&mut self, field: Field, value: Value) {
        if let Err(problem) = self.check(field, &value) {
            return self.fail(problem);
        }
        let label = format!("{}: {}", field.label(), shown(&value));
        self.edit(&label, |editor| store(field, &value, editor));
    }

    pub fn set_autostart(&mut self, entries: Vec<String>) {
        self.edit("Inicialização", |editor| editor.set_list("autostart", &entries));
    }

    fn check(&self, field: Field, value: &Value) -> Result<(), String> {
        let text = |field: Field| match read(field, &self.config) {
            Value::Text(text) => text,
            Value::Number(_) | Value::Flag(_) => String::new(),
        };
        let checked = match (field, value) {
            (Field::Layout, Value::Text(layout)) => self.context.keymaps.check(layout, &text(Field::Variant)),
            (Field::Variant, Value::Text(variant)) => self.context.keymaps.check(&text(Field::Layout), variant),
            _ => Ok(()),
        };
        checked.map_err(|unknown| unknown.to_string())
    }

    fn edit(&mut self, what: &str, change: impl FnOnce(&mut Editor)) {
        let Source::Editable(editor) = &self.source else {
            return self.fail("o sparky.toml não pôde ser lido; corrija o arquivo antes de mudar algo");
        };
        let mut draft = editor.clone();
        change(&mut draft);
        let contents = draft.contents();
        let config = match parse(&contents) {
            Ok(config) => config,
            Err(error) => return self.fail(format!("o sparky.toml ficaria inválido: {error}")),
        };
        if let Err(error) = self.services.store.save(&contents) {
            return self.fail(format!("não deu para salvar: {error}"));
        }
        self.source = Source::Editable(draft);
        self.config = config;
        self.announce(what);
    }

    fn announce(&mut self, what: &str) {
        match self.services.compositor.reload() {
            Ok(()) => self.inform(format!("{what} (salvo e aplicado)")),
            Err(CompositorError::Unreachable(_)) => self.inform(format!("{what} (salvo; vale quando o sparky iniciar)")),
            Err(refused) => self.fail(refused),
        }
    }
}
