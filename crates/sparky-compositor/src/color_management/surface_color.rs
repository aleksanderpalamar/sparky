use std::sync::Arc;

use smithay::reexports::wayland_server::DisplayHandle;
use smithay::wayland::compositor::Cacheable;

use super::records::Record;

#[derive(Debug, Clone, Default)]
pub struct SurfaceColor {
    pub description: Option<Arc<Record>>,
}

impl Cacheable for SurfaceColor {
    fn commit(&mut self, _: &DisplayHandle) -> Self {
        self.clone()
    }

    fn merge_into(self, into: &mut Self, _: &DisplayHandle) {
        *into = self;
    }
}

#[cfg(test)]
mod tests {
    use smithay::reexports::wayland_server::Display;
    use smithay::wayland::compositor::Cacheable;

    use super::SurfaceColor;
    use crate::color_management::description::{
        ImageDescription, NamedPrimaries, TransferFunction,
    };
    use crate::color_management::records::Records;

    #[test]
    fn descricao_pendente_so_vale_no_commit_e_continua_depois()
    -> Result<(), Box<dyn std::error::Error>> {
        let display = Display::<()>::new()?;
        let handle = display.handle();
        let mut records = Records::default();
        let pq = records.record(ImageDescription::standard(
            TransferFunction::St2084Pq,
            NamedPrimaries::Bt2020,
        ));
        let mut pending = SurfaceColor::default();
        let mut current = SurfaceColor::default();
        pending.description = Some(pq.clone());
        assert!(current.description.is_none());
        pending.commit(&handle).merge_into(&mut current, &handle);
        assert_eq!(
            current.description.as_ref().map(|record| record.identity),
            Some(pq.identity)
        );
        pending.commit(&handle).merge_into(&mut current, &handle);
        assert_eq!(
            current.description.as_ref().map(|record| record.identity),
            Some(pq.identity)
        );
        pending.description = None;
        pending.commit(&handle).merge_into(&mut current, &handle);
        assert!(current.description.is_none());
        Ok(())
    }
}
