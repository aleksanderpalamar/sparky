use crate::config::settings::{Layout, Logo};
use crate::render::view::View;
use crate::render::visual::segment::{Line, Segment};

const GAP: usize = 3;
const MIN_INFO: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arrangement {
    Beside,
    Above,
    Alone,
}

impl Arrangement {
    pub(crate) fn choose(view: &View, logo_width: usize) -> Arrangement {
        match (view.logo, view.layout) {
            (Logo::Hide, _) => Arrangement::Alone,
            _ if logo_width > view.width => Arrangement::Alone,
            (Logo::Show, Layout::Stacked) => Arrangement::Above,
            (Logo::Show, Layout::Classic) if logo_width + GAP + MIN_INFO > view.width => {
                Arrangement::Above
            }
            (Logo::Show, Layout::Classic) => Arrangement::Beside,
        }
    }

    pub(crate) fn info_room(self, width: usize, logo_width: usize) -> usize {
        match self {
            Arrangement::Beside => width.saturating_sub(logo_width + GAP),
            Arrangement::Above | Arrangement::Alone => width,
        }
    }
}

pub(crate) fn compose(arrangement: Arrangement, logo: &[Line], info: Vec<Line>) -> Vec<Line> {
    match arrangement {
        Arrangement::Beside => beside(logo, info),
        Arrangement::Above => above(logo, info),
        Arrangement::Alone => info,
    }
}

fn beside(logo: &[Line], info: Vec<Line>) -> Vec<Line> {
    let column = logo.iter().map(Line::width).max().unwrap_or(0) + GAP;
    let height = logo.len().max(info.len());
    let mut info = info.into_iter();
    (0..height)
        .map(|index| {
            joined(
                logo.get(index).cloned().unwrap_or_default(),
                info.next(),
                column,
            )
        })
        .collect()
}

fn joined(left: Line, right: Option<Line>, column: usize) -> Line {
    let Some(right) = right.filter(|right| right.width() > 0) else {
        return left;
    };
    let pad = Segment::space(column - left.width());
    left.then(Line::new(vec![pad])).then(right)
}

fn above(logo: &[Line], info: Vec<Line>) -> Vec<Line> {
    let separator = (!info.is_empty()).then(Line::blank);
    logo.iter().cloned().chain(separator).chain(info).collect()
}
