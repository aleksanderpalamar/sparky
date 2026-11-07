use std::slice;
use std::sync::Arc;

use crate::domain::field::Field;
use crate::domain::status::{Status, Value};
use crate::linux::host::Host;
use crate::sources::Source;

pub type Collect = fn(&dyn Host) -> Result<Value, Status>;

pub struct Probe {
    field: &'static Field,
    host: Arc<dyn Host>,
    collect: Collect,
}

impl Probe {
    pub fn new(field: &'static Field, host: Arc<dyn Host>, collect: Collect) -> Self {
        Self {
            field,
            host,
            collect,
        }
    }
}

impl Source for Probe {
    fn fields(&self) -> &'static [Field] {
        slice::from_ref(self.field)
    }

    fn collect(&self, wanted: &[Field]) -> Vec<(Field, Status)> {
        if !wanted.contains(self.field) {
            return Vec::new();
        }
        let status = (self.collect)(self.host.as_ref()).map_or_else(|status| status, Status::ready);
        vec![(*self.field, status)]
    }
}
