use sparky_config::ResourceMode;
use sparky_resources::domain::Switch;
use sparky_resources::protocol::Request;

use crate::app::state::Settings;
use crate::domain::performance::{Knob, Performance, mode, mode_key};

const TABLE: &str = "resources";

impl Settings {
    pub fn refresh_performance(&mut self) {
        let confirming = self.performance.confirming;
        self.performance = Performance::load(self.services.resources.as_ref());
        self.performance.confirming = confirming;
    }

    pub fn toggle_performance(&mut self, knob: Knob) {
        let (key, on, request): (&str, bool, fn(Switch) -> Request) = match knob {
            Knob::Enabled => ("enabled", !self.config.resources.enabled, |switch| {
                Request::SetEnabled { switch }
            }),
            Knob::Learning => ("learning", !self.config.resources.learning, |switch| {
                Request::SetLearning { switch }
            }),
        };
        let label = crate::i18n::text(&format!("performance-{key}"));
        self.edit(&label, |editor| editor.set_flag(TABLE, key, on));
        self.tell(request(if on { Switch::On } else { Switch::Off }));
    }

    pub fn choose_performance_mode(&mut self, chosen: ResourceMode) {
        let key = mode_key(chosen);
        let label = crate::i18n::text(&format!("performance-mode-{key}"));
        self.edit(&label, |editor| editor.set_text(TABLE, "mode", Some(key)));
        self.tell(Request::SetMode { mode: mode(chosen) });
    }

    pub fn forget_performance(&mut self, confirmed: bool) {
        if !confirmed {
            self.performance.confirming = true;
            return;
        }
        self.performance.confirming = false;
        match self.services.resources.send(Request::ForgetKnowledge) {
            Ok(()) => self.inform(crate::i18n::text("performance-forgotten")),
            Err(reason) => self.fail(crate::i18n::with(
                "performance-unreachable",
                "reason",
                &reason,
            )),
        }
    }

    pub fn cancel_performance(&mut self) {
        self.performance.confirming = false;
    }

    fn tell(&mut self, request: Request) {
        if self.services.resources.send(request).is_ok() {
            self.refresh_performance();
        }
    }
}
