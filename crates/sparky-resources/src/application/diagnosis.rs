use super::encoding::{HORIZONS, OUTPUTS, Sample};
use super::validation::Errors;
use crate::domain::{Activity, METRICS};

pub const ENTRY: f64 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segment {
    Desktop,
    Entering,
    Playing,
    Other,
}

pub const SEGMENTS: [Segment; 4] = [
    Segment::Desktop,
    Segment::Entering,
    Segment::Playing,
    Segment::Other,
];

impl Segment {
    pub fn label(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Entering => "entrada",
            Self::Playing => "jogo",
            Self::Other => "outros",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Phase {
    since: Option<f64>,
}

impl Phase {
    pub fn segment(&mut self, sample: &Sample) -> Segment {
        if !sample.activities.contains(Activity::Gaming) {
            self.since = None;
            return match sample.activities.is_sensitive() {
                true => Segment::Other,
                false => Segment::Desktop,
            };
        }
        let since = *self.since.get_or_insert(sample.at);
        match sample.at - since < ENTRY {
            true => Segment::Entering,
            false => Segment::Playing,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Cell {
    pub count: usize,
    pub neural: f64,
    pub statistical: f64,
}

impl Cell {
    fn mean(total: f64, count: usize) -> f64 {
        total / count.max(1) as f64
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tally {
    cells: Vec<Cell>,
}

impl Default for Tally {
    fn default() -> Self {
        Self {
            cells: vec![Cell::default(); SEGMENTS.len() * OUTPUTS],
        }
    }
}

impl Tally {
    pub fn add(&mut self, segment: Segment, output: usize, neural: f64, statistical: f64) {
        if let Some(cell) = self.cells.get_mut(index(segment, output)) {
            cell.count += 1;
            cell.neural += neural;
            cell.statistical += statistical;
        }
    }

    pub fn cell(&self, segment: Segment, output: usize) -> Cell {
        self.cells
            .get(index(segment, output))
            .copied()
            .unwrap_or_default()
    }

    pub fn errors(&self, neural: bool) -> Errors {
        let total = self.cells.iter().fold(Cell::default(), |sum, cell| Cell {
            count: sum.count + cell.count,
            neural: sum.neural + cell.neural,
            statistical: sum.statistical + cell.statistical,
        });
        Errors {
            statistical: Cell::mean(total.statistical, total.count),
            neural: neural.then(|| Cell::mean(total.neural, total.count)),
            count: total.count,
        }
    }

    pub fn lines(&self) -> Vec<String> {
        SEGMENTS
            .iter()
            .flat_map(|segment| (0..OUTPUTS).map(move |output| (*segment, output)))
            .filter_map(|(segment, output)| line(segment, output, self.cell(segment, output)))
            .collect()
    }
}

fn index(segment: Segment, output: usize) -> usize {
    segment as usize * OUTPUTS + output
}

fn line(segment: Segment, output: usize, cell: Cell) -> Option<String> {
    let metric = METRICS.get(output % METRICS.len())?;
    let horizon = HORIZONS.get(output / METRICS.len())?;
    (cell.count > 0).then(|| {
        format!(
            "{} {metric:?} {horizon}s n={} rede={:.2} estat={:.2}",
            segment.label(),
            cell.count,
            Cell::mean(cell.neural, cell.count),
            Cell::mean(cell.statistical, cell.count)
        )
    })
}
