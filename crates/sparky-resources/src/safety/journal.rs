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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    Refused(ExecError),
    Unverified(Outcome),
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
        outcome => Err(Failure::Unverified(outcome)),
    }
}

pub fn restore(executor: &impl Executor, applied: &Applied) -> Outcome {
    if executor.tasks(applied.adjustment.target).is_err() {
        return Outcome::Gone;
    }
    let kind = applied.adjustment.change.kind();
    let mut outcome = Outcome::Restored;
    for (task, previous) in &applied.previous {
        match executor.read(*task, kind) {
            Err(ExecError::Gone) => continue,
            Ok(current) if current == *previous => continue,
            Ok(current) if current != applied.adjustment.change => {
                outcome = Outcome::ChangedElsewhere
            }
            Ok(_) => {
                let back = executor.write(*task, previous).is_ok()
                    && executor.read(*task, kind).is_ok_and(|now| now == *previous);
                if !back {
                    return Outcome::Unconfirmed;
                }
            }
            Err(_) => return Outcome::Unconfirmed,
        }
    }
    outcome
}
