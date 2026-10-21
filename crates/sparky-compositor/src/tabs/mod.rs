pub mod fit;
pub mod lettering;
pub mod paint;
pub mod slots;

use sparky_tiling::WindowId;

use crate::state::Sparky;
use crate::windows::traits::title_of;

impl Sparky {
    pub fn title(&self, window: WindowId) -> String {
        self.windows.get(window).map(title_of).unwrap_or_default()
    }
}
