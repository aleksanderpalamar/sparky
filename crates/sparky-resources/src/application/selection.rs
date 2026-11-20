use super::diagnosis::Tally;
use super::encoding::Sample;
use super::plan::Plan;
use super::progress::{Outcome, outcome};
use super::validation::{Errors, tally, validate};
use crate::neural::Hybrid;

pub enum Judged {
    Promoted(Hybrid, Errors),
    Rejected(Errors, Outcome, Option<Hybrid>),
}

pub fn best(
    previous: Option<Hybrid>,
    fresh: Hybrid,
    rows: &[Sample],
    split: usize,
) -> (Hybrid, Tally) {
    let measured = tally(rows, split, Some(&fresh));
    let Some(previous) = previous else {
        return (fresh, measured);
    };
    let earlier = tally(rows, split, Some(&previous));
    match score(&earlier) < score(&measured) {
        true => (previous, earlier),
        false => (fresh, measured),
    }
}

pub fn judge(
    plan: &Plan,
    model: Hybrid,
    rows: &[Sample],
    split: usize,
    active: Option<&Hybrid>,
    candidate: Errors,
) -> Judged {
    let current = active.and_then(|model| validate(rows, split, Some(model)).neural);
    let finite = model.params.is_finite();
    match outcome(plan.margin, &candidate, current, finite) {
        Outcome::Promoted => Judged::Promoted(model, candidate),
        Outcome::Unstable => Judged::Rejected(candidate, Outcome::Unstable, None),
        rejected => Judged::Rejected(candidate, rejected, Some(model)),
    }
}

fn score(tally: &Tally) -> f64 {
    tally
        .errors(true)
        .neural
        .filter(|value| value.is_finite())
        .unwrap_or(f64::INFINITY)
}
