use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Slot<T>(Arc<Mutex<Option<T>>>);

impl<T> Slot<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn put(&self, value: T) {
        if let Ok(mut held) = self.0.lock() {
            *held = Some(value);
        }
    }

    pub fn take(&self) -> Option<T> {
        self.0.lock().ok().and_then(|mut held| held.take())
    }
}

impl<T> Clone for Slot<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Slot;

    #[test]
    fn o_valor_guardado_numa_copia_aparece_na_outra_uma_vez() {
        let slot = Slot::new();
        slot.clone().put(7);
        assert_eq!(slot.take(), Some(7));
        assert_eq!(slot.take(), None);
    }

    #[test]
    fn sem_valor_guardado_fica_vazio() {
        assert_eq!(Slot::<i32>::new().take(), None);
    }
}
