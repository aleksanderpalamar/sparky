use sparky_tiling::WindowId;

pub struct Registry<T> {
    next: u64,
    entries: Vec<(WindowId, T)>,
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            next: 1,
            entries: Vec::new(),
        }
    }
}

impl<T> Registry<T> {
    pub fn insert(&mut self, item: T) -> WindowId {
        let id = WindowId(self.next);
        self.next += 1;
        self.entries.push((id, item));
        id
    }

    pub fn remove(&mut self, id: WindowId) -> Option<T> {
        let position = self.entries.iter().position(|(known, _)| *known == id)?;
        Some(self.entries.remove(position).1)
    }

    pub fn get(&self, id: WindowId) -> Option<&T> {
        self.entries.iter().find(|(known, _)| *known == id).map(|(_, item)| item)
    }

    pub fn find(&self, matches: impl Fn(&T) -> bool) -> Option<(WindowId, &T)> {
        self.entries
            .iter()
            .find(|(_, item)| matches(item))
            .map(|(id, item)| (*id, item))
    }

    pub fn iter(&self) -> impl Iterator<Item = (WindowId, &T)> {
        self.entries.iter().map(|(id, item)| (*id, item))
    }
}

#[cfg(test)]
mod tests {
    use super::Registry;
    use sparky_tiling::WindowId;

    #[test]
    fn cada_janela_ganha_um_numero_novo() {
        let mut registry = Registry::default();
        assert_eq!(registry.insert("kitty"), WindowId(1));
        assert_eq!(registry.insert("firefox"), WindowId(2));
    }

    #[test]
    fn o_numero_de_janela_removida_nao_volta() {
        let mut registry = Registry::default();
        let first = registry.insert("kitty");
        registry.remove(first);
        assert_eq!(registry.insert("foot"), WindowId(2));
    }

    #[test]
    fn acha_pelo_numero_e_por_condicao() {
        let mut registry = Registry::default();
        registry.insert("kitty");
        let firefox = registry.insert("firefox");
        assert_eq!(registry.get(firefox), Some(&"firefox"));
        assert_eq!(registry.find(|name| name.starts_with('k')).map(|(id, _)| id), Some(WindowId(1)));
    }

    #[test]
    fn remover_devolve_o_item() {
        let mut registry = Registry::default();
        let id = registry.insert("kitty");
        assert_eq!(registry.remove(id), Some("kitty"));
        assert_eq!(registry.get(id), None);
        assert_eq!(registry.remove(id), None);
    }
}
