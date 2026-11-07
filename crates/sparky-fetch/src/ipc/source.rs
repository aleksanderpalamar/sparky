use crate::domain::field::Field;
use crate::domain::status::Status;
use crate::ipc::fields::status;
use crate::ipc::port::Compositor;
use crate::sources::Source;

const FIELDS: &[Field] = &[
    Field::Compositor,
    Field::Session,
    Field::Monitors,
    Field::Resolution,
    Field::Theme,
    Field::Colors,
];

pub struct SparkySource<C: Compositor> {
    compositor: C,
}

impl<C: Compositor> SparkySource<C> {
    pub fn new(compositor: C) -> Self {
        Self { compositor }
    }
}

impl<C: Compositor> Source for SparkySource<C> {
    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn collect(&self, wanted: &[Field]) -> Vec<(Field, Status)> {
        let snapshot = self.compositor.snapshot();
        FIELDS
            .iter()
            .filter(|field| wanted.contains(field))
            .map(|field| (*field, status(*field, &snapshot)))
            .collect()
    }
}
