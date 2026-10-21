mod commit;
mod compositor;
mod decoration;
mod dmabuf;
mod fractional_scale;
mod layer_shell;
mod pointer;
mod seat;
mod selection;
mod xdg_shell;

use smithay::wayland::output::OutputHandler;
use smithay::{delegate_output, delegate_viewporter, delegate_xwayland_shell};

use crate::state::Sparky;

impl OutputHandler for Sparky {}

delegate_output!(Sparky);
delegate_viewporter!(Sparky);
delegate_xwayland_shell!(Sparky);
