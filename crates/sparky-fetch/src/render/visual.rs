mod ansi;
mod fit;
mod info;
mod layout;
mod logo;
mod segment;
mod style;
mod swatch;

#[cfg(test)]
mod ansi_tests;
#[cfg(test)]
mod fit_tests;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod info_tests;
#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod logo_tests;
#[cfg(test)]
mod segment_tests;
#[cfg(test)]
mod style_tests;
#[cfg(test)]
mod swatch_tests;
#[cfg(test)]
mod visual_tests;

use crate::domain::report::Report;
use crate::render::view::View;
use layout::Arrangement;
use style::Paint;

pub fn render(report: &Report, view: &View) -> String {
    let paint = Paint::resolve(report, view);
    let arrangement = Arrangement::choose(view, logo::width());
    let room = arrangement.info_room(view.width, logo::width());
    let info = info::block(report, &paint, room);
    layout::compose(arrangement, &logo::lines(), info)
        .into_iter()
        .map(|line| fit::clip(line.trimmed(), view.width))
        .map(|line| paint.line(&line) + "\n")
        .collect()
}
