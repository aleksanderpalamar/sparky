use crate::domain::pinned::Pinned;
use crate::ports::PinnedStore;

pub fn restore(store: &dyn PinnedStore) -> Pinned {
    match store.load() {
        Ok(Some(lines)) => {
            let written = lines.iter().any(|line| !line.trim().is_empty());
            let pinned = Pinned::from_ids(lines);
            match written && pinned.ids().is_empty() {
                true => Pinned::default(),
                false => pinned,
            }
        }
        Ok(None) => Pinned::default(),
        Err(error) => {
            eprintln!("sparky-dock: configuração ilegível, usando a padrão: {error}");
            Pinned::default()
        }
    }
}
