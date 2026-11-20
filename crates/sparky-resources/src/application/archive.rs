use super::encoding::Sample;
use super::progress::{Outcome, Round};
use crate::domain::{Activities, Features, METRICS};

const SAMPLE: usize = METRICS.len() + 2;
const ROUND: usize = 4;
const OUTCOMES: [Outcome; 5] = [
    Outcome::Promoted,
    Outcome::BehindStatistics,
    Outcome::BehindActive,
    Outcome::Unstable,
    Outcome::NoData,
];
pub const GAP: f64 = 600.0;

pub fn pack_samples<'a>(samples: impl Iterator<Item = &'a Sample>) -> Vec<f32> {
    samples
        .flat_map(|sample| {
            let metrics = METRICS
                .iter()
                .map(|metric| sample.features.get(*metric).unwrap_or(f32::NAN));
            std::iter::once(sample.at as f32)
                .chain(metrics)
                .chain(std::iter::once(f32::from(sample.activities.bits())))
        })
        .collect()
}

pub fn unpack_samples(values: &[f32], before: f64) -> Vec<Sample> {
    if !values.len().is_multiple_of(SAMPLE) {
        return Vec::new();
    }
    let samples: Vec<Sample> = values.chunks_exact(SAMPLE).map_while(sample).collect();
    let ordered = samples.windows(2).all(|pair| pair[0].at <= pair[1].at);
    let Some(last) = samples.last().map(|sample| sample.at).filter(|_| ordered) else {
        return Vec::new();
    };
    let shift = before - GAP - last;
    samples
        .into_iter()
        .map(|sample| Sample {
            at: sample.at + shift,
            ..sample
        })
        .collect()
}

pub fn pack_rounds(rounds: u32, history: &[Round]) -> Vec<f32> {
    let entries = history.iter().flat_map(|round| {
        [
            round.at_seconds as f32,
            code(round.outcome),
            round.neural.map_or(f32::NAN, |value| value as f32),
            round.statistical as f32,
        ]
    });
    std::iter::once(rounds as f32).chain(entries).collect()
}

pub fn unpack_rounds(values: &[f32]) -> Option<(u32, Vec<Round>)> {
    let (rounds, entries) = values.split_first()?;
    if !entries.len().is_multiple_of(ROUND) || !rounds.is_finite() || *rounds < 0.0 {
        return None;
    }
    let history = entries
        .chunks_exact(ROUND)
        .map(round)
        .collect::<Option<Vec<_>>>()?;
    Some((*rounds as u32, history))
}

fn sample(chunk: &[f32]) -> Option<Sample> {
    let (at, rest) = chunk.split_first()?;
    let (bits, metrics) = rest.split_last()?;
    if !at.is_finite() || !bits.is_finite() {
        return None;
    }
    let features = METRICS
        .iter()
        .zip(metrics)
        .fold(Features::default(), |features, (metric, value)| {
            features.with(*metric, value.is_finite().then_some(*value))
        });
    Some(Sample {
        at: f64::from(*at),
        features,
        activities: Activities::from_bits(*bits as u16),
    })
}

fn round(chunk: &[f32]) -> Option<Round> {
    let [at, outcome, neural, statistical] = chunk else {
        return None;
    };
    let outcome = OUTCOMES.get(*outcome as usize).copied()?;
    let valid = at.is_finite() && *at >= 0.0 && statistical.is_finite();
    valid.then(|| Round {
        at_seconds: *at as u64,
        outcome,
        neural: neural.is_finite().then_some(f64::from(*neural)),
        statistical: f64::from(*statistical),
    })
}

fn code(outcome: Outcome) -> f32 {
    OUTCOMES
        .iter()
        .position(|known| *known == outcome)
        .map_or(f32::NAN, |index| index as f32)
}
