#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    pub path: String,
    pub size: String,
    pub label: String,
    pub filesystem: String,
    pub mountpoint: Option<String>,
    pub removable: bool,
}

impl Volume {
    pub fn name(&self) -> &str {
        match self.label.is_empty() {
            true => &self.path,
            false => &self.label,
        }
    }

    pub fn mounted(&self) -> bool {
        self.mountpoint.is_some()
    }

    pub fn external(&self) -> bool {
        let point = self.mountpoint.as_deref().unwrap_or_default();
        self.removable || point.starts_with("/run/media/") || point.starts_with("/media/")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    List,
    Mount(String),
    Unmount(String),
}

impl Operation {
    pub fn command(&self) -> Option<(&str, &str)> {
        match self {
            Self::List => None,
            Self::Mount(device) => Some(("mount", device)),
            Self::Unmount(device) => Some(("unmount", device)),
        }
    }
}
