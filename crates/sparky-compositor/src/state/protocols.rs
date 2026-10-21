use smithay::input::SeatState;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::cursor_shape::CursorShapeManagerState;
use smithay::wayland::dmabuf::DmabufState;
use smithay::wayland::fractional_scale::FractionalScaleManagerState;
use smithay::wayland::output::OutputManagerState;
use smithay::wayland::pointer_constraints::PointerConstraintsState;
use smithay::wayland::relative_pointer::RelativePointerManagerState;
use smithay::wayland::selection::data_device::DataDeviceState;
use smithay::wayland::selection::primary_selection::PrimarySelectionState;
use smithay::wayland::selection::wlr_data_control::DataControlState;
use smithay::wayland::session_lock::SessionLockManagerState;
use smithay::wayland::shell::wlr_layer::WlrLayerShellState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shell::xdg::decoration::XdgDecorationState;
use smithay::wayland::shm::ShmState;
use smithay::wayland::viewporter::ViewporterState;
use smithay::wayland::xwayland_shell::XWaylandShellState;

use crate::screencopy::ScreencopyState;
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
    pub dmabuf: DmabufState,
    pub xwayland_shell: XWaylandShellState,
    pub cursor_shape: CursorShapeManagerState,
    pub relative_pointer: RelativePointerManagerState,
    pub pointer_constraints: PointerConstraintsState,
    pub fractional_scale: FractionalScaleManagerState,
    pub screencopy: ScreencopyState,
    pub session_lock: SessionLockManagerState,
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
            dmabuf: DmabufState::new(),
            xwayland_shell: XWaylandShellState::new::<Sparky>(display),
            cursor_shape: CursorShapeManagerState::new::<Sparky>(display),
            relative_pointer: RelativePointerManagerState::new::<Sparky>(display),
            pointer_constraints: PointerConstraintsState::new::<Sparky>(display),
            fractional_scale: FractionalScaleManagerState::new::<Sparky>(display),
            screencopy: ScreencopyState::new::<Sparky>(display),
            session_lock: SessionLockManagerState::new::<Sparky, _>(display, |_| true),
        }
    }
}
