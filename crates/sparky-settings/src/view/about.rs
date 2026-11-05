use iced::Element;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::hardware::memory_label;
use crate::i18n;
use crate::view::kit::controls::reading;
use crate::view::kit::rows::{Line, grouped};

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let about = &settings.context.about;
    let known = |value: &Option<String>| value.clone().unwrap_or_else(|| i18n::text("unknown"));
    let gpus = match about.hardware.gpus.is_empty() {
        true => i18n::text("unknown"),
        false => about.hardware.gpus.join("\n"),
    };
    let facts = [
        (
            "about-sparky",
            "about-version",
            env!("CARGO_PKG_VERSION").to_owned(),
        ),
        ("about-system", "about-system", known(&about.distribution)),
        ("about-system", "about-kernel", known(&about.kernel)),
        ("about-system", "about-computer", known(&about.host)),
        (
            "about-hardware",
            "about-processor",
            known(&about.hardware.processor),
        ),
        ("about-hardware", "about-gpu", gpus),
        (
            "about-hardware",
            "about-ram",
            known(&about.hardware.memory_kib.map(memory_label)),
        ),
    ];
    let rows = facts.into_iter().map(|(group, label, value)| {
        (
            i18n::text(group),
            Line::new(i18n::text(label), reading(skin, value)).view(skin),
        )
    });
    grouped(skin, rows)
}
