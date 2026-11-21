use serde::{Deserialize, Serialize};

use crate::domain::{Adjustment, Change};
use crate::ports::{ExecError, Executor};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Applied {
    pub adjustment: Adjustment,
    pub previous: Vec<(u32, Change)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Restored,
    Gone,
    ChangedElsewhere,
    Unconfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    Refused(ExecError),
    Unverified(Outcome, Box<Applied>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Origin {
    Known,
    Born,
}

enum Step {
    Done,
    Elsewhere,
    Unconfirmed,
}

pub fn apply(executor: &impl Executor, adjustment: &Adjustment) -> Result<Applied, Failure> {
    let tasks = executor
        .tasks(adjustment.target)
        .map_err(Failure::Refused)?;
    let kind = adjustment.change.kind();
    let previous = tasks
        .iter()
        .map(|task| executor.read(*task, kind).map(|change| (*task, change)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(Failure::Refused)?;
    if !previous
        .iter()
        .all(|(_, before)| eases(before, &adjustment.change))
    {
        return Err(Failure::Refused(ExecError::Unsupported));
    }
    let applied = Applied {
        adjustment: adjustment.clone(),
        previous,
    };
    let written = tasks
        .iter()
        .try_for_each(|task| executor.write(*task, &adjustment.change));
    let verified = written.is_ok()
        && tasks.iter().all(|task| {
            executor
                .read(*task, kind)
                .is_ok_and(|now| now == adjustment.change)
        });
    if verified {
        return Ok(applied);
    }
    match restore(executor, &applied) {
        Outcome::Restored | Outcome::Gone => {
            Err(Failure::Refused(written.err().unwrap_or(ExecError::Failed)))
        }
        outcome => Err(Failure::Unverified(outcome, Box::new(applied))),
    }
}

pub fn restore(executor: &impl Executor, applied: &Applied) -> Outcome {
    let Ok(tasks) = executor.tasks(applied.adjustment.target) else {
        return Outcome::Gone;
    };
    let mut outcome = Outcome::Restored;
    for (task, previous, origin) in pending(applied, &tasks) {
        match rewind(executor, applied, task, &previous, origin) {
            Step::Done => {}
            Step::Elsewhere => outcome = Outcome::ChangedElsewhere,
            Step::Unconfirmed => return Outcome::Unconfirmed,
        }
    }
    outcome
}

fn pending(applied: &Applied, tasks: &[u32]) -> Vec<(u32, Change, Origin)> {
    let known = applied
        .previous
        .iter()
        .map(|(task, previous)| (*task, previous.clone(), Origin::Known));
    let born = baseline(applied).into_iter().flat_map(|base| {
        tasks
            .iter()
            .filter(|task| !applied.previous.iter().any(|(known, _)| known == *task))
            .map(move |task| (*task, base.clone(), Origin::Born))
    });
    known.chain(born).collect()
}

fn baseline(applied: &Applied) -> Option<&Change> {
    let main = applied.adjustment.target.pid;
    applied
        .previous
        .iter()
        .find(|(task, _)| *task == main)
        .or_else(|| applied.previous.first())
        .map(|(_, previous)| previous)
}

fn rewind(
    executor: &impl Executor,
    applied: &Applied,
    task: u32,
    previous: &Change,
    origin: Origin,
) -> Step {
    match executor.read(task, previous.kind()) {
        Err(ExecError::Gone) => Step::Done,
        Ok(current) if current == *previous => Step::Done,
        Ok(current) if current != applied.adjustment.change => match origin {
            Origin::Known => Step::Elsewhere,
            Origin::Born => Step::Done,
        },
        Ok(_) if wrote_back(executor, task, previous) => Step::Done,
        _ => Step::Unconfirmed,
    }
}

fn wrote_back(executor: &impl Executor, task: u32, previous: &Change) -> bool {
    executor.write(task, previous).is_ok()
        && executor
            .read(task, previous.kind())
            .is_ok_and(|now| now == *previous)
}

fn eases(before: &Change, change: &Change) -> bool {
    match (before, change) {
        (Change::Nice(before), Change::Nice(value)) => (0..*value).contains(before),
        _ => true,
    }
}
