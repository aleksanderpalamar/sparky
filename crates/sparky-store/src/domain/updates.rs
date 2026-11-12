use super::{Installation, Manager, valid_id};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateSource {
    Flatpak(Installation),
    Native(Manager),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateKind {
    Application,
    Runtime,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateItem {
    pub id: String,
    pub name: String,
    pub installed: Option<String>,
    pub available: Option<String>,
    pub source: UpdateSource,
    pub kind: UpdateKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateGroup {
    pub source: UpdateSource,
    pub items: Vec<UpdateItem>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateReport {
    pub groups: Vec<UpdateGroup>,
    pub warning: Option<String>,
}

impl UpdateReport {
    pub fn count(&self) -> usize {
        self.groups.iter().map(|group| group.items.len()).sum()
    }

    pub fn targets(&self) -> Vec<UpdateTarget> {
        self.groups
            .iter()
            .filter(|group| !group.items.is_empty())
            .map(|group| match group.source {
                UpdateSource::Flatpak(scope) => UpdateTarget::Flatpak {
                    scope,
                    reference: None,
                },
                UpdateSource::Native(manager) => UpdateTarget::Native {
                    manager,
                    package: None,
                },
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateTarget {
    Flatpak {
        scope: Installation,
        reference: Option<String>,
    },
    Native {
        manager: Manager,
        package: Option<String>,
    },
}

impl UpdateItem {
    pub fn target(&self) -> UpdateTarget {
        match self.source {
            UpdateSource::Flatpak(scope) => UpdateTarget::Flatpak {
                scope,
                reference: Some(self.id.clone()),
            },
            UpdateSource::Native(manager) => UpdateTarget::Native {
                manager,
                package: (manager != Manager::Pacman).then(|| self.id.clone()),
            },
        }
    }
}

pub fn valid_flatpak_ref(reference: &str) -> bool {
    let parts: Vec<_> = reference.split('/').collect();
    matches!(parts.as_slice(), ["app" | "runtime", id, arch, branch]
        if valid_id(id) && valid_id(arch) && valid_id(branch))
}

pub fn valid_native_package(package: &str) -> bool {
    if valid_id(package) {
        return true;
    }
    package
        .split_once(':')
        .is_some_and(|(name, architecture)| valid_id(name) && valid_id(architecture))
}

#[cfg(test)]
mod tests {
    use super::{
        UpdateGroup, UpdateItem, UpdateKind, UpdateReport, UpdateSource, UpdateTarget,
        valid_flatpak_ref,
    };
    use crate::domain::{Installation, Manager};

    #[test]
    fn referencias_flatpak_precisam_ser_completas_e_seguras() {
        assert!(valid_flatpak_ref("app/org.gimp.GIMP/x86_64/stable"));
        assert!(valid_flatpak_ref(
            "runtime/org.freedesktop.Platform/x86_64/24.08"
        ));
        assert!(!valid_flatpak_ref("--system"));
        assert!(!valid_flatpak_ref("app/org.gimp.GIMP/x86_64"));
        assert!(!valid_flatpak_ref("app/org.gimp.GIMP/x86_64/../evil"));
    }

    #[test]
    fn atualizacao_arch_de_um_item_abrange_o_sistema() {
        let item = UpdateItem {
            id: "mesa".into(),
            name: "mesa".into(),
            installed: None,
            available: None,
            source: UpdateSource::Native(Manager::Pacman),
            kind: UpdateKind::System,
        };
        assert_eq!(
            item.target(),
            UpdateTarget::Native {
                manager: Manager::Pacman,
                package: None
            }
        );
    }

    #[test]
    fn plano_agrupa_transacoes_por_origem() {
        let report = UpdateReport {
            groups: vec![
                UpdateGroup {
                    source: UpdateSource::Flatpak(Installation::User),
                    items: vec![UpdateItem {
                        id: "app/org.gimp.GIMP/x86_64/stable".into(),
                        name: "GIMP".into(),
                        installed: None,
                        available: None,
                        source: UpdateSource::Flatpak(Installation::User),
                        kind: UpdateKind::Application,
                    }],
                    error: None,
                },
                UpdateGroup {
                    source: UpdateSource::Native(Manager::Pacman),
                    items: vec![UpdateItem {
                        id: "mesa".into(),
                        name: "mesa".into(),
                        installed: None,
                        available: None,
                        source: UpdateSource::Native(Manager::Pacman),
                        kind: UpdateKind::System,
                    }],
                    error: None,
                },
            ],
            warning: None,
        };
        assert_eq!(report.count(), 2);
        assert_eq!(report.targets().len(), 2);
    }
}
