use std::collections::{BTreeMap, BTreeSet};

use crate::ports::{Filesystems, SystemFiles};

#[derive(Default)]
pub struct Arvore {
    files: BTreeMap<String, String>,
    links: BTreeMap<String, String>,
}

impl Arvore {
    pub fn com(mut self, path: &str, content: &str) -> Self {
        self.files.insert(path.to_owned(), content.to_owned());
        self
    }

    pub fn link(mut self, path: &str, target: &str) -> Self {
        self.links.insert(path.to_owned(), target.to_owned());
        self
    }
}

impl SystemFiles for Arvore {
    fn read(&self, path: &str) -> Option<String> {
        self.files.get(path).cloned()
    }

    fn list(&self, path: &str) -> Vec<String> {
        let prefix = format!("{path}/");
        let names: BTreeSet<String> = self
            .files
            .keys()
            .chain(self.links.keys())
            .filter_map(|key| {
                key.strip_prefix(&prefix)?
                    .split('/')
                    .next()
                    .map(str::to_owned)
            })
            .collect();
        names.into_iter().collect()
    }

    fn link(&self, path: &str) -> Option<String> {
        self.links.get(path).cloned()
    }
}

pub struct Discos;

impl Filesystems for Discos {
    fn usage(&self, mount: &str) -> Option<(u64, u64)> {
        match mount {
            "/" => Some((1000, 400)),
            "/dados" => Some((500, 100)),
            _ => None,
        }
    }
}
