use iced::Element;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::hardware::memory_label;
use crate::view::kit::controls::reading;
use crate::view::kit::rows::{Line, grouped};

const UNKNOWN: &str = "não informado";

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let about = &settings.context.about;
    let known = |value: &Option<String>| value.clone().unwrap_or_else(|| UNKNOWN.to_owned());
    let gpus = match about.hardware.gpus.is_empty() {
        true => UNKNOWN.to_owned(),
        false => about.hardware.gpus.join("\n"),
    };
    let facts = [
        ("Sparky", "Versão", env!("CARGO_PKG_VERSION").to_owned()),
        ("Sistema", "Sistema", known(&about.distribution)),
        ("Sistema", "Kernel", known(&about.kernel)),
        ("Sistema", "Computador", known(&about.host)),
        ("Hardware", "Processador", known(&about.hardware.processor)),
        ("Hardware", "GPU", gpus),
        (
            "Hardware",
            "RAM",
            known(&about.hardware.memory_kib.map(memory_label)),
        ),
    ];
    let rows = facts
        .into_iter()
        .map(|(group, label, value)| (group, Line::new(label, reading(skin, value)).view(skin)));
    grouped(skin, rows)
}
