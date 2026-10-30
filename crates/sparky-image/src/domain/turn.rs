use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Turn {
    #[default]
    Upright,
    Right,
    Half,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spin {
    Clockwise,
    CounterClockwise,
}

impl Turn {
    pub(super) fn spin(self, spin: Spin) -> Self {
        match (self, spin) {
            (Turn::Upright, Spin::Clockwise) | (Turn::Half, Spin::CounterClockwise) => Turn::Right,
            (Turn::Right, Spin::Clockwise) | (Turn::Left, Spin::CounterClockwise) => Turn::Half,
            (Turn::Half, Spin::Clockwise) | (Turn::Upright, Spin::CounterClockwise) => Turn::Left,
            (Turn::Left, Spin::Clockwise) | (Turn::Right, Spin::CounterClockwise) => Turn::Upright,
        }
    }

    pub(super) fn sideways(self) -> bool {
        matches!(self, Turn::Right | Turn::Left)
    }

    pub fn radians(self) -> f32 {
        match self {
            Turn::Upright => 0.0,
            Turn::Right => FRAC_PI_2,
            Turn::Half => PI,
            Turn::Left => 3.0 * FRAC_PI_2,
        }
    }
}
