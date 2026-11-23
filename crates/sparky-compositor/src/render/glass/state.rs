use smithay::backend::renderer::element::Id;
use smithay::backend::renderer::utils::CommitCounter;
use smithay::utils::{Buffer, Logical, Rectangle, Size};

#[derive(Debug)]
pub struct GlassState {
    pub id: Id,
    pub commit: CommitCounter,
    region: Vec<Rectangle<i32, Logical>>,
    src: Size<f64, Buffer>,
}

impl Default for GlassState {
    fn default() -> Self {
        Self {
            id: Id::new(),
            commit: CommitCounter::default(),
            region: Vec::new(),
            src: Size::default(),
        }
    }
}

impl GlassState {
    pub fn touch(&mut self, region: &[Rectangle<i32, Logical>], src: Size<f64, Buffer>) {
        if self.region == region && self.src == src {
            return;
        }
        self.region = region.to_vec();
        self.src = src;
        self.commit.increment();
    }
}
