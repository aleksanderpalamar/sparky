use std::time::Instant;

use super::backoff::Backoff;
use super::log::{Decision, Log, Verdict};
use super::planner::{Candidate, Contention, Purpose, plan};
use crate::domain::{Activities, Authority, Change};
use crate::policy::{Block, Capabilities, Policy, Situation};
use crate::ports::{Executor, Storage};
use crate::safety::Ledger;
use crate::safety::{Applied, commit, prepare};
use crate::telemetry::consumers::Consumer;

pub struct Moment<'a> {
    pub at: Instant,
    pub seconds: u64,
    pub activities: Activities,
    pub contention: Contention,
    pub consumers: &'a [(Consumer, u64)],
    pub protected: &'a [u32],
    pub authority: Authority,
    pub confidence: f32,
}

pub(super) struct Active {
    pub(super) applied: Applied,
    pub(super) purpose: Purpose,
    pub(super) process: String,
    pub(super) before: f32,
    pub(super) since: Instant,
    pub(super) kept: bool,
}

pub struct Conductor<E, S> {
    pub(super) executor: E,
    pub(super) ledger: Ledger<S>,
    pub(super) policy: Policy,
    pub(super) backoff: Backoff,
    pub(super) capabilities: Capabilities,
    pub(super) spare: Vec<u32>,
    pub(super) active: Option<Active>,
    pub(super) log: Log,
    pub(super) last: Option<(String, Verdict)>,
    pub(super) unconfirmed: bool,
}

impl<E: Executor, S: Storage> Conductor<E, S> {
    pub fn new(
        executor: E,
        ledger: Ledger<S>,
        capabilities: Capabilities,
        spare: Vec<u32>,
    ) -> Self {
        Self {
            executor,
            ledger,
            policy: Policy::default(),
            backoff: Backoff::default(),
            capabilities,
            spare,
            active: None,
            log: Log::default(),
            last: None,
            unconfirmed: false,
        }
    }

    pub fn tick(&mut self, moment: &Moment) {
        if let Some(active) = self.active.take() {
            self.active = self.follow(active, moment);
            return;
        }
        let eligible = self.backoff.eligible(moment.consumers, moment.at);
        let Some(candidate) = plan(
            moment.activities,
            moment.contention,
            &eligible,
            moment.protected,
            &self.spare,
        ) else {
            return;
        };
        if moment.authority == Authority::None {
            return self.note(
                moment,
                &candidate,
                candidate
                    .options
                    .first()
                    .map(|option| option.change.clone()),
                Verdict::Simulated,
            );
        }
        let situation = Situation {
            authority: moment.authority,
            telemetry: true,
            protected: moment.protected,
            capabilities: self.capabilities,
            now: moment.at,
        };
        let mut block = Block::NoAuthority;
        for option in &candidate.options {
            match self.policy.decide(option, moment.confidence, &situation) {
                Ok(()) => return self.start(moment, &candidate, option),
                Err(reason) => block = reason,
            }
        }
        let change = candidate.options.last().map(|option| option.change.clone());
        self.note(moment, &candidate, change, Verdict::Blocked(block));
    }

    fn start(
        &mut self,
        moment: &Moment,
        candidate: &Candidate,
        option: &crate::domain::Adjustment,
    ) {
        let applied = match prepare(&self.executor, option) {
            Ok(intent) => intent,
            Err(failure) => return self.failed(moment, candidate, option.change.clone(), failure),
        };
        if let Err(error) = self.ledger.record(applied.clone()) {
            eprintln!("sparky-resources: intenção não registrada, ajuste cancelado: {error}");
            return self.note(
                moment,
                candidate,
                Some(option.change.clone()),
                Verdict::Failed,
            );
        }
        if let Err(failure) = commit(&self.executor, &applied) {
            self.abandon(&applied, &failure);
            return self.failed(moment, candidate, option.change.clone(), failure);
        }
        let before = moment.contention.stall(candidate.purpose).unwrap_or(0.0);
        self.last = None;
        self.push(
            moment,
            candidate.purpose,
            candidate.name.clone(),
            option.change.clone(),
            Verdict::Applied,
        );
        self.active = Some(Active {
            applied,
            purpose: candidate.purpose,
            process: candidate.name.clone(),
            before,
            since: moment.at,
            kept: false,
        });
    }

    pub(super) fn note(
        &mut self,
        moment: &Moment,
        candidate: &Candidate,
        change: Option<Change>,
        verdict: Verdict,
    ) {
        let repeated = self.last.as_ref() == Some(&(candidate.name.clone(), verdict.clone()));
        if let (false, Some(change)) = (repeated, change) {
            self.last = Some((candidate.name.clone(), verdict.clone()));
            self.push(
                moment,
                candidate.purpose,
                candidate.name.clone(),
                change,
                verdict,
            );
        }
    }

    pub(super) fn push(
        &mut self,
        moment: &Moment,
        purpose: Purpose,
        process: String,
        change: Change,
        verdict: Verdict,
    ) {
        let stall = moment.contention.stall(purpose);
        self.log.push(Decision {
            at_seconds: moment.seconds,
            activities: moment.activities,
            purpose,
            process,
            change,
            stall,
            verdict,
        });
    }
}
