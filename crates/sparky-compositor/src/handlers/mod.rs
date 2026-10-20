mod commit;
mod compositor;
mod decoration;
mod dmabuf;
mod layer_shell;
mod seat;
mod selection;
mod xdg_shell;

use smithay::wayland::output::OutputHandler;
use smithay::{delegate_output, delegate_viewporter};

use crate::state::Sparky;

impl OutputHandler for Sparky {}

delegate_output!(Sparky);
delegate_viewporter!(Sparky);
