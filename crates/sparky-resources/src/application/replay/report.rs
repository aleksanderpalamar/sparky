use super::confirm::Paired;
use super::resample::{Interval, interval, mean};
use crate::application::diagnosis::Segment;
use crate::application::encoding::HORIZONS;
use crate::domain::METRICS;

const BLOCK: usize = 30;
const DRAWS: usize = 1000;
const MATERIAL: f64 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub count: usize,
    pub neural: Option<f64>,
    pub statistical: f64,
    pub persistence: Option<f64>,
    pub gain: Option<Interval>,
    pub margin: Option<f64>,
    pub regression: bool,
}

pub fn summarize(pairs: &[Paired], seed: u64) -> Summary {
    let statistical: Vec<f64> = pairs.iter().map(|pair| pair.statistical).collect();
    let neural: Option<Vec<f64>> = pairs.iter().map(|pair| pair.neural).collect();
    let persistence: Option<Vec<f64>> = pairs.iter().map(|pair| pair.persistence).collect();
    let gains: Vec<f64> = pairs
        .iter()
        .filter_map(|pair| pair.neural.map(|neural| pair.statistical - neural))
        .collect();
    let joint = instants(pairs);
    let gain = (joint.len() >= 2 * BLOCK)
        .then(|| interval(&joint, BLOCK, DRAWS, seed))
        .flatten();
    let baseline = mean(&statistical);
    let lost = -mean(&gains);
    Summary {
        count: pairs.len(),
        neural: neural.map(|values| mean(&values)),
        statistical: baseline,
        persistence: persistence.map(|values| mean(&values)),
        gain,
        margin: (!gains.is_empty() && baseline > 0.0).then(|| -lost / baseline),
        regression: lost >= MATERIAL && gain.is_some_and(|gain| gain.high < 0.0),
    }
}

fn instants(pairs: &[Paired]) -> Vec<f64> {
    let mut timed: Vec<(f64, f64)> = pairs
        .iter()
        .filter_map(|pair| {
            pair.neural
                .map(|neural| (pair.at, pair.statistical - neural))
        })
        .collect();
    timed.sort_by(|left, right| left.0.total_cmp(&right.0));
    timed
        .chunk_by(|left, right| left.0 == right.0)
        .map(|group| group.iter().map(|(_, gain)| gain).sum::<f64>() / group.len() as f64)
        .collect()
}

pub fn line(segment: Segment, output: usize, summary: &Summary) -> String {
    let metric = METRICS[output % METRICS.len()];
    let horizon = HORIZONS[output / METRICS.len()];
    let show =
        |value: Option<f64>| value.map_or_else(|| "-".to_owned(), |value| format!("{value:.3}"));
    let gain = summary.gain.map_or_else(
        || "-".to_owned(),
        |gain| format!("{:.3} [{:.3}; {:.3}]", gain.mean, gain.low, gain.high),
    );
    format!(
        "{},{metric:?},{horizon},{},{},{:.3},{},{gain},{},{}",
        segment.label(),
        summary.count,
        show(summary.neural),
        summary.statistical,
        show(summary.persistence),
        show(summary.margin.map(|margin| margin * 100.0)),
        if summary.regression { "regressao" } else { "-" },
    )
}
