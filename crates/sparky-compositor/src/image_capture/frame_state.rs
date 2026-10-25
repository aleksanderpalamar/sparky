use super::rules::valid_damage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    NoBuffer,
    AlreadyCaptured,
    InvalidDamage,
}

#[derive(Default)]
enum Phase {
    #[default]
    Collecting,
    Captured,
}

pub struct FrameState<T> {
    phase: Phase,
    buffer: Option<T>,
}

impl<T> Default for FrameState<T> {
    fn default() -> Self {
        Self {
            phase: Phase::Collecting,
            buffer: None,
        }
    }
}

impl<T> FrameState<T> {
    pub fn attach(&mut self, buffer: T) -> Result<(), FrameError> {
        self.collecting()?;
        self.buffer = Some(buffer);
        Ok(())
    }

    pub fn capture(&mut self) -> Result<T, FrameError> {
        self.collecting()?;
        let buffer = self.buffer.take().ok_or(FrameError::NoBuffer)?;
        self.phase = Phase::Captured;
        Ok(buffer)
    }

    pub fn damage(&self, x: i32, y: i32, width: i32, height: i32) -> Result<(), FrameError> {
        self.collecting()?;
        match valid_damage(x, y, width, height) {
            true => Ok(()),
            false => Err(FrameError::InvalidDamage),
        }
    }

    fn collecting(&self) -> Result<(), FrameError> {
        match self.phase {
            Phase::Collecting => Ok(()),
            Phase::Captured => Err(FrameError::AlreadyCaptured),
        }
    }
}
