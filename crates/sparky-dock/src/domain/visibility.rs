use super::layout::Visibility;
use super::presence::Seen;

pub struct Moment<'a> {
    pub autohide: bool,
    pub windows: &'a [Seen],
    pub hovering: bool,
    pub busy: bool,
}

pub fn visibility(moment: &Moment<'_>) -> Visibility {
    let covered = moment.windows.iter().any(|seen| seen.visible);
    match moment.autohide && covered && !moment.hovering && !moment.busy {
        true => Visibility::Hidden,
        false => Visibility::Shown,
    }
}
