use sparky_theme::Rgb;

use crate::config::settings::{Colors, Format, Layout, Logo};
use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::{Reason, Status, Value};
use crate::render::view::View;

pub(crate) const ACCENT: Rgb = Rgb::new(0xbb, 0x9a, 0xf7);

pub(crate) fn view(width: usize, color: bool) -> View {
    View {
        format: Format::Visual,
        color,
        logo: Logo::Show,
        layout: Layout::Classic,
        width,
        colors: Colors::default(),
    }
}

pub(crate) fn palette() -> Vec<Rgb> {
    vec![
        ACCENT,
        Rgb::new(0xf7, 0x76, 0x8e),
        Rgb::new(0x9e, 0xce, 0x6a),
        Rgb::new(0xe0, 0xaf, 0x68),
        Rgb::new(0x7a, 0xa2, 0xf7),
    ]
}

pub(crate) fn report() -> Report {
    Report {
        entries: vec![
            Entry::new(Field::Os, Status::text("Arch Linux")),
            Entry::new(Field::Kernel, Status::text("6.17.1-arch1-1")),
            Entry::new(Field::Uptime, Status::text("3 h 12 min")),
            Entry::new(
                Field::Cpu,
                Status::text("AMD Ryzen 7 5800X (16) @ 4.85 GHz"),
            ),
            Entry::new(Field::Gpu, Status::unavailable(Reason::Missing)),
            Entry::new(Field::Memory, Status::text("6.2 GiB / 31.3 GiB")),
            Entry::new(Field::Packages, Status::TimedOut),
            Entry::new(Field::Compositor, Status::text("Sparky 0.30.0")),
            Entry::new(Field::Monitors, monitors()),
            Entry::new(Field::Theme, Status::text("Tokyo Night")),
            Entry::new(Field::Colors, Status::ready(Value::Colors(palette()))),
        ],
    }
}

pub(crate) fn monitors() -> Status {
    Status::lines(vec![
        "DP-1 2560x1440 @ 165 Hz".into(),
        "HDMI-A-1 1920x1080 @ 60 Hz".into(),
    ])
}
