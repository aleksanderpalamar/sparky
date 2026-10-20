use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::delegate_dmabuf;
use smithay::wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier};

use crate::state::Sparky;

impl DmabufHandler for Sparky {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.protocols.dmabuf
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, dmabuf: Dmabuf, notifier: ImportNotifier) {
        match self.import_dmabuf(&dmabuf) {
            true => {
                if notifier.successful::<Sparky>().is_err() {
                    tracing::warn!("cliente sumiu antes de receber o buffer de GPU");
                }
            }
            false => notifier.failed(),
        }
    }
}

delegate_dmabuf!(Sparky);
