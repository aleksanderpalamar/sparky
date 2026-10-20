use smithay::desktop::layer_map_for_output;
use smithay::output::Output;
use smithay::utils::{Logical, Rectangle};
use sparky_tiling::{Rect, Screen};

use crate::state::Sparky;

impl Sparky {
    pub fn sync_outputs(&mut self) {
        let outputs: Vec<Output> = self.space.outputs().cloned().collect();
        for output in &outputs {
            self.sync_output(output);
        }
        let names: Vec<String> = outputs.iter().map(Output::name).collect();
        let gone: Vec<String> = self
            .desktop
            .monitors()
            .iter()
            .filter(|monitor| !names.contains(&monitor.name))
            .map(|monitor| monitor.name.clone())
            .collect();
        for name in gone {
            self.desktop.remove_monitor(&name);
        }
    }

    fn sync_output(&mut self, output: &Output) {
        let Some(geometry) = self.space.output_geometry(output) else {
            return;
        };
        let zone = {
            let mut map = layer_map_for_output(output);
            map.arrange();
            map.non_exclusive_zone()
        };
        let full = rect_of(geometry);
        let area = Rect::new(
            geometry.loc.x + zone.loc.x,
            geometry.loc.y + zone.loc.y,
            zone.size.w,
            zone.size.h,
        );
        let name = output.name();
        self.desktop.add_monitor(&name, full);
        self.desktop.set_screen(&name, Screen { full, area });
    }
}

pub fn rect_of(rectangle: Rectangle<i32, Logical>) -> Rect {
    Rect::new(rectangle.loc.x, rectangle.loc.y, rectangle.size.w, rectangle.size.h)
}
