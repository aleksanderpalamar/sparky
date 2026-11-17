#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Group {
    pub name: String,
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Groups {
    list: Vec<Group>,
}

impl Groups {
    pub fn new(list: Vec<Group>) -> Self {
        let mut groups = Self::default();
        for group in list {
            if let Some(index) = groups.create(&group.name) {
                group.apps.iter().for_each(|app| {
                    groups.add(index, app);
                });
            }
        }
        groups
    }

    pub fn list(&self) -> &[Group] {
        &self.list
    }

    pub fn get(&self, index: usize) -> Option<&Group> {
        self.list.get(index)
    }

    pub fn create(&mut self, name: &str) -> Option<usize> {
        let name = self.free(name, None)?;
        self.list.push(Group {
            name,
            apps: Vec::new(),
        });
        Some(self.list.len() - 1)
    }

    pub fn rename(&mut self, index: usize, name: &str) -> bool {
        let Some(name) = self.free(name, Some(index)) else {
            return false;
        };
        let Some(group) = self.list.get_mut(index) else {
            return false;
        };
        group.name = name;
        true
    }

    pub fn remove(&mut self, index: usize) -> Option<Group> {
        (index < self.list.len()).then(|| self.list.remove(index))
    }

    pub fn add(&mut self, index: usize, app: &str) -> bool {
        match self.list.get_mut(index) {
            Some(group) if !group.apps.iter().any(|held| held == app) => {
                group.apps.push(app.to_owned());
                true
            }
            _ => false,
        }
    }

    pub fn take(&mut self, index: usize, app: &str) -> bool {
        let Some(group) = self.list.get_mut(index) else {
            return false;
        };
        let before = group.apps.len();
        group.apps.retain(|held| held != app);
        group.apps.len() != before
    }

    pub fn holds(&self, index: usize, app: &str) -> bool {
        self.get(index)
            .is_some_and(|group| group.apps.iter().any(|held| held == app))
    }

    pub fn grouped(&self, app: &str) -> bool {
        self.list
            .iter()
            .any(|group| group.apps.iter().any(|held| held == app))
    }

    fn free(&self, name: &str, except: Option<usize>) -> Option<String> {
        let name = clean(name);
        let taken = self.list.iter().enumerate().any(|(index, group)| {
            Some(index) != except && group.name.to_lowercase() == name.to_lowercase()
        });
        (!name.is_empty() && !taken).then_some(name)
    }
}

fn clean(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}
