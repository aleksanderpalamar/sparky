use serde::Deserialize;

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
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct Listing {
    blockdevices: Vec<Node>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct Node {
    name: String,
    path: String,
    size: String,
    #[serde(rename = "type")]
    kind: String,
    mountpoint: Option<String>,
    label: Option<String>,
    rm: bool,
    hotplug: bool,
    fstype: Option<String>,
    children: Vec<Node>,
}

pub fn parse(output: &str) -> Result<Vec<Volume>, serde_json::Error> {
    let listing: Listing = serde_json::from_str(output)?;
    let mut volumes = Vec::new();
    for node in &listing.blockdevices {
        collect(node, false, &mut volumes);
    }
    Ok(volumes)
}

fn collect(node: &Node, parent_removable: bool, volumes: &mut Vec<Volume>) {
    let removable = parent_removable || node.rm || node.hotplug;
    for child in &node.children {
        collect(child, removable, volumes);
    }
    if !eligible(node, removable) {
        return;
    }
    volumes.push(Volume {
        path: node.path.clone(),
        size: node.size.clone(),
        label: node.label.clone().unwrap_or_default(),
        filesystem: node.fstype.clone().unwrap_or_default(),
        mountpoint: node.mountpoint.clone().filter(|point| !point.is_empty()),
        removable,
    });
}

fn eligible(node: &Node, removable: bool) -> bool {
    if !matches!(node.kind.as_str(), "disk" | "part") || !node.path.starts_with("/dev/") || node.name.starts_with("zram") {
        return false;
    }
    if !node.fstype.as_deref().is_some_and(|format| !format.is_empty() && format != "swap") {
        return false;
    }
    let point = node.mountpoint.as_deref().unwrap_or_default();
    if matches!(point, "/" | "/boot" | "/home" | "/usr" | "/var") || point.starts_with("/boot/") {
        return false;
    }
    removable || point.starts_with("/run/media/") || point.starts_with("/media/")
}
