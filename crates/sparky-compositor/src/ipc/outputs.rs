use sparky_config::OutputScale;
use sparky_ipc::OutputInfo;

use crate::layout::rotation::rotation;
use crate::layout::screens::{Screen, output_mode};
use crate::state::Sparky;

impl Sparky {
    pub fn output_infos(&self) -> Vec<OutputInfo> {
        let focused = self
            .desktop
            .focused_monitor()
            .map(|monitor| monitor.name.clone());
        let screens = self.connected_screens();
        let mut infos: Vec<OutputInfo> = self
            .desktop
            .monitors()
            .iter()
            .map(|monitor| {
                let screen = screens.iter().find(|screen| screen.name == monitor.name);
                let mut info = OutputInfo {
                    name: monitor.name.clone(),
                    x: monitor.screen.full.x,
                    y: monitor.screen.full.y,
                    width: monitor.screen.full.width,
                    height: monitor.screen.full.height,
                    focused: focused.as_deref() == Some(monitor.name.as_str()),
                    ..details(screen)
                };
                if let Some(output) = self
                    .space
                    .outputs()
                    .find(|output| output.name() == monitor.name)
                {
                    info.mode = output.current_mode().map(output_mode);
                    info.scale =
                        OutputScale::from_factor(output.current_scale().fractional_scale())
                            .unwrap_or_default();
                    info.rotation = rotation(output.current_transform());
                }
                info
            })
            .collect();
        let disabled = screens.iter().filter(|screen| !screen.enabled);
        infos.extend(disabled.map(|screen| details(Some(screen))));
        infos
    }
}

fn details(screen: Option<&Screen>) -> OutputInfo {
    let Some(screen) = screen else {
        return OutputInfo::default();
    };
    OutputInfo {
        name: screen.name.clone(),
        description: screen.description.clone(),
        modes: screen.modes.clone(),
        vrr: screen.vrr,
        enabled: screen.enabled,
        ..OutputInfo::default()
    }
}
