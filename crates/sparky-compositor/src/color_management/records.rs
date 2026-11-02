use std::collections::HashMap;
use std::sync::{Arc, Weak};

use super::description::ImageDescription;

#[derive(Debug)]
pub struct Record {
    pub identity: u64,
    pub description: ImageDescription,
}

#[derive(Debug)]
pub struct Records {
    next: u64,
    live: HashMap<ImageDescription, Weak<Record>>,
}

impl Default for Records {
    fn default() -> Self {
        Self {
            next: 1,
            live: HashMap::new(),
        }
    }
}

impl Records {
    pub fn record(&mut self, description: ImageDescription) -> Arc<Record> {
        if let Some(record) = self.live.get(&description).and_then(Weak::upgrade) {
            return record;
        }
        self.live.retain(|_, record| record.strong_count() > 0);
        let record = Arc::new(Record {
            identity: self.next,
            description,
        });
        self.next += 1;
        self.live.insert(description, Arc::downgrade(&record));
        record
    }
}

#[cfg(test)]
mod tests {
    use super::Records;
    use crate::color_management::description::{
        ImageDescription, NamedPrimaries, TransferFunction,
    };

    fn srgb() -> ImageDescription {
        ImageDescription::standard(TransferFunction::Srgb, NamedPrimaries::Srgb)
    }

    fn pq() -> ImageDescription {
        ImageDescription::standard(TransferFunction::St2084Pq, NamedPrimaries::Bt2020)
    }

    #[test]
    fn descricoes_iguais_vivas_compartilham_a_identidade() {
        let mut records = Records::default();
        let primeira = records.record(srgb());
        let copia = records.record(srgb());
        let outra = records.record(pq());
        assert_eq!(primeira.identity, copia.identity);
        assert_ne!(primeira.identity, outra.identity);
        assert_ne!(primeira.identity, 0);
    }

    #[test]
    fn identidade_nunca_e_reaproveitada() {
        let mut records = Records::default();
        let antiga = records.record(srgb()).identity;
        let nova = records.record(srgb()).identity;
        assert!(nova > antiga);
    }

    #[test]
    fn registros_sem_dono_saem_da_tabela() {
        let mut records = Records::default();
        let viva = records.record(srgb());
        drop(records.record(pq()));
        let _ = records.record(ImageDescription::standard(
            TransferFunction::Gamma22,
            NamedPrimaries::DisplayP3,
        ));
        assert_eq!(records.live.len(), 2);
        assert_eq!(records.record(srgb()).identity, viva.identity);
    }
}
