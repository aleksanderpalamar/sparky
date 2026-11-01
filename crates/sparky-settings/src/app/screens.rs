use crate::app::state::Settings;
use crate::domain::screens::{ScreenChange, changed};

impl Settings {
    pub fn change_screen(&mut self, name: &str, change: ScreenChange) {
        let outputs = match &self.outputs {
            Ok(outputs) => outputs.clone(),
            Err(error) => return self.fail(error.to_string()),
        };
        let Some(info) = outputs.iter().find(|output| output.name == name) else {
            return;
        };
        let rule = match changed(info, &self.config.output, &outputs, change) {
            Ok(rule) => rule,
            Err(problem) => return self.fail(problem),
        };
        self.edit(&format!("Monitor {name}"), |editor| {
            editor.set_output(&rule)
        });
        self.outputs = self.services.compositor.outputs();
    }
}
