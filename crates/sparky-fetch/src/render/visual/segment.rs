use sparky_theme::Rgb;

use crate::render::visual::ansi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    Plain,
    Logo,
    Label,
    Value,
    Swatch(Rgb),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Segment {
    pub(crate) text: String,
    pub(crate) tone: Tone,
}

impl Segment {
    pub(crate) fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
        }
    }

    pub(crate) fn space(width: usize) -> Self {
        Self::new(" ".repeat(width), Tone::Plain)
    }

    pub(crate) fn width(&self) -> usize {
        ansi::width(&self.text)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Line {
    pub(crate) segments: Vec<Segment>,
}

impl Line {
    pub(crate) fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }

    pub(crate) fn blank() -> Self {
        Self::default()
    }

    pub(crate) fn width(&self) -> usize {
        self.segments.iter().map(Segment::width).sum()
    }

    pub(crate) fn then(mut self, other: Line) -> Line {
        self.segments.extend(other.segments);
        self
    }

    pub(crate) fn trimmed(mut self) -> Line {
        while let Some(last) = self.segments.last_mut() {
            let kept = last.text.trim_end().len();
            last.text.truncate(kept);
            if !last.text.is_empty() {
                break;
            }
            self.segments.pop();
        }
        self
    }
}
