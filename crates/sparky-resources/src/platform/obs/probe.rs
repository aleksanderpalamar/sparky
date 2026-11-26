use std::time::{Duration, Instant};

use super::client::capturing;
use super::setup::{Setup, load};
use crate::context::{Sensed, Unavailable};
use crate::ports::CaptureSensor;

const EVERY: Duration = Duration::from_secs(5);

pub struct ObsProbe {
    fixed: Option<Setup>,
    last: Option<(Instant, Sensed)>,
}

impl ObsProbe {
    pub fn from_disk() -> Self {
        Self {
            fixed: None,
            last: None,
        }
    }

    pub fn with_setup(setup: Setup) -> Self {
        Self {
            fixed: Some(setup),
            last: None,
        }
    }

    pub fn sense(&mut self, now: Instant) -> Sensed {
        if let Some((at, sensed)) = self.last
            && now.duration_since(at) < EVERY
        {
            return sensed;
        }
        let setup = self.fixed.clone().unwrap_or_else(load);
        let sensed = probe(&setup);
        self.last = Some((now, sensed));
        sensed
    }
}

impl CaptureSensor for ObsProbe {
    fn sense(&mut self, now: Instant) -> Sensed {
        ObsProbe::sense(self, now)
    }
}

fn probe(setup: &Setup) -> Sensed {
    let Setup::Enabled(server) = setup else {
        return Sensed::Unavailable(Unavailable::Disabled);
    };
    match capturing(server) {
        Ok(true) => Sensed::Capturing,
        Ok(false) => Sensed::Idle,
        Err(reason) => Sensed::Unavailable(reason),
    }
}
