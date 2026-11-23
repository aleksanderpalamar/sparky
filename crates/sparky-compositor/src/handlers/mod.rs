mod commit;
mod compositor;
mod decoration;
mod dmabuf;
mod fractional_scale;
mod idle;
mod layer_shell;
mod pointer;
mod seat;
mod selection;
mod session_lock;
mod xdg_shell;

use smithay::wayland::output::OutputHandler;

use crate::state::Sparky;

impl OutputHandler for Sparky {}

smithay::delegate_dispatch2!(Sparky);
