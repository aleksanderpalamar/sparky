use sparky_theme::{Palette, Rgb};

use crate::config::settings::Colors;
use crate::domain::report::Report;
use crate::render::view::View;
use crate::render::visual::ansi;
use crate::render::visual::segment::{Line, Segment, Tone};
use crate::render::visual::swatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Inks {
    pub(crate) accent: Rgb,
    pub(crate) label: Rgb,
    pub(crate) value: Option<Rgb>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Paint {
    Plain,
    TrueColor(Inks),
}

impl Inks {
    fn resolve(report: &Report, colors: Colors) -> Inks {
        let accent = colors
            .accent
            .or_else(|| theme_accent(report))
            .unwrap_or(Palette::TOKYO_NIGHT.accent);
        Inks {
            accent,
            label: colors.label.unwrap_or(accent),
            value: colors.value,
        }
    }

    fn of(self, tone: Tone) -> Option<Rgb> {
        match tone {
            Tone::Plain => None,
            Tone::Logo => Some(self.accent),
            Tone::Label => Some(self.label),
            Tone::Value => self.value,
            Tone::Swatch(color) => Some(color),
        }
    }
}

impl Paint {
    pub(crate) fn resolve(report: &Report, view: &View) -> Paint {
        match view.color {
            true => Paint::TrueColor(Inks::resolve(report, view.colors)),
            false => Paint::Plain,
        }
    }

    pub(crate) fn line(&self, line: &Line) -> String {
        line.segments
            .iter()
            .map(|segment| self.segment(segment))
            .collect()
    }

    fn segment(&self, segment: &Segment) -> String {
        match self.ink(segment.tone) {
            Some(color) if !segment.text.is_empty() => ansi::paint(&segment.text, color),
            _ => segment.text.clone(),
        }
    }

    fn ink(&self, tone: Tone) -> Option<Rgb> {
        match self {
            Paint::Plain => None,
            Paint::TrueColor(inks) => inks.of(tone),
        }
    }
}

fn theme_accent(report: &Report) -> Option<Rgb> {
    swatch::palette(report)?.first().copied()
}
