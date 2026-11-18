use super::conductor::{Active, Conductor, Moment};
use super::evaluation::{Judgement, judge};
use super::log::Verdict;
use crate::domain::Activity;
use crate::ports::{Executor, Storage};
use crate::safety::{Outcome, restore};

impl<E: Executor, S: Storage> Conductor<E, S> {
    pub fn release(&mut self) -> bool {
        let Some(active) = self.active.take() else {
            return true;
        };
        self.finish(&active) != Outcome::Unconfirmed
    }

    pub(super) fn follow(&mut self, mut active: Active, moment: &Moment) -> Option<Active> {
        let sensitive = moment.activities.contains(Activity::Gaming)
            || moment.activities.contains(Activity::Recording);
        let withdrawn = moment.authority == crate::domain::Authority::None;
        if !sensitive || withdrawn {
            let outcome = self.finish(&active);
            self.record(moment, &active, Verdict::Finished(outcome));
            return None;
        }
        if active.kept {
            return Some(active);
        }
        match judge(
            active.before,
            moment.contention.stall(active.purpose),
            moment.at.duration_since(active.since),
        ) {
            Judgement::Waiting => Some(active),
            Judgement::Keep => {
                active.kept = true;
                self.backoff.worked(&active.process);
                self.record(moment, &active, Verdict::Kept);
                Some(active)
            }
            Judgement::Revert => {
                self.backoff.failed(&active.process, moment.at);
                let outcome = self.finish(&active);
                self.record(moment, &active, Verdict::Reverted(outcome));
                None
            }
        }
    }

    pub(super) fn finish(&mut self, active: &Active) -> Outcome {
        let outcome = restore(&self.executor, &active.applied);
        self.unconfirmed |= outcome == Outcome::Unconfirmed;
        if outcome != Outcome::Unconfirmed
            && let Err(error) = self.ledger.settle(active.applied.adjustment.target)
        {
            eprintln!("sparky-resources: registro de ajustes não atualizado: {error}");
        }
        outcome
    }

    fn record(&mut self, moment: &Moment, active: &Active, verdict: Verdict) {
        let change = active.applied.adjustment.change.clone();
        self.push(
            moment,
            active.purpose,
            active.process.clone(),
            change,
            verdict,
        );
    }
}
