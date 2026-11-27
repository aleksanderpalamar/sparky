use sparky_tiling::{Fullscreen, Layer, Placement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stratum {
    Overlay,
    Unmanaged,
    Top,
    Windows,
    Bottom,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cover {
    Open,
    Fullscreen,
}

const OPEN: [Stratum; 6] = [
    Stratum::Overlay,
    Stratum::Unmanaged,
    Stratum::Top,
    Stratum::Windows,
    Stratum::Bottom,
    Stratum::Background,
];

const FULLSCREEN: [Stratum; 5] = [
    Stratum::Overlay,
    Stratum::Unmanaged,
    Stratum::Windows,
    Stratum::Bottom,
    Stratum::Background,
];

pub fn strata(cover: Cover) -> &'static [Stratum] {
    match cover {
        Cover::Open => &OPEN,
        Cover::Fullscreen => &FULLSCREEN,
    }
}

#[derive(Debug, Default)]
pub struct Stacking {
    covered: Vec<String>,
}

impl Stacking {
    pub fn refresh(&mut self, arrangement: &[(String, Vec<Placement>)]) {
        self.covered = arrangement
            .iter()
            .filter(|(_, placements)| placements.iter().any(covers))
            .map(|(name, _)| name.clone())
            .collect();
    }

    pub fn cover(&self, output: &str) -> Cover {
        match self.covered.iter().any(|name| name == output) {
            true => Cover::Fullscreen,
            false => Cover::Open,
        }
    }
}

pub fn covers(placement: &Placement) -> bool {
    placement.layer == Layer::Fullscreen(Fullscreen::Monitor)
}

#[cfg(test)]
#[path = "stacking_tests.rs"]
mod tests;
