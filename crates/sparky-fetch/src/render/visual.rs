use crate::domain::report::Report;
use crate::render::plain;
use crate::render::view::View;

pub fn render(report: &Report, _view: &View) -> String {
    plain::render(report)
}
