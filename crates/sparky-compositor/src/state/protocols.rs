use smithay::input::SeatState;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::output::OutputManagerState;
use smithay::wayland::selection::data_device::DataDeviceState;
use smithay::wayland::selection::primary_selection::PrimarySelectionState;
use smithay::wayland::selection::wlr_data_control::DataControlState;
use smithay::wayland::shell::wlr_layer::WlrLayerShellState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shell::xdg::decoration::XdgDecorationState;
use smithay::wayland::shm::ShmState;
use smithay::wayland::viewporter::ViewporterState;

use crate::state::Sparky;

pub struct Protocols {
    pub compositor: CompositorState,
    pub xdg_shell: XdgShellState,
    pub layer_shell: WlrLayerShellState,
    pub decoration: XdgDecorationState,
    pub shm: ShmState,
    pub outputs: OutputManagerState,
    pub seat: SeatState<Sparky>,
    pub data_device: DataDeviceState,
    pub primary_selection: PrimarySelectionState,
    pub data_control: DataControlState,
    pub viewporter: ViewporterState,
}

impl Protocols {
    pub fn new(display: &DisplayHandle) -> Self {
        let primary_selection = PrimarySelectionState::new::<Sparky>(display);
        let data_control = DataControlState::new::<Sparky, _>(display, Some(&primary_selection), |_| true);
        Self {
            compositor: CompositorState::new::<Sparky>(display),
            xdg_shell: XdgShellState::new::<Sparky>(display),
            layer_shell: WlrLayerShellState::new::<Sparky>(display),
            decoration: XdgDecorationState::new::<Sparky>(display),
            shm: ShmState::new::<Sparky>(display, vec![]),
            outputs: OutputManagerState::new_with_xdg_output::<Sparky>(display),
            seat: SeatState::new(),
            data_device: DataDeviceState::new::<Sparky>(display),
            primary_selection,
            data_control,
            viewporter: ViewporterState::new::<Sparky>(display),
        }
    }
}
