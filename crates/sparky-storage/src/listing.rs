use std::collections::BTreeSet;
use std::path::{Component, Path};

use serde::Deserialize;

use crate::Volume;

#[derive(Deserialize, Default)]
#[serde(default)]
struct Listing {
    blockdevices: Vec<Node>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Selection {
    Available,
    External,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Node {
    name: String,
    path: String,
    size: String,
    #[serde(rename = "type")]
    kind: String,
    mountpoint: Option<String>,
    mountpoints: Vec<Option<String>>,
    label: Option<String>,
    rm: bool,
    hotplug: bool,
    fstype: Option<String>,
    children: Vec<Node>,
}

pub fn parse(output: &str) -> Result<Vec<Volume>, serde_json::Error> {
    parse_with(output, Selection::Available)
}

pub fn external_volumes(output: &str) -> Result<Vec<Volume>, serde_json::Error> {
    Ok(parse_with(output, Selection::External)?
        .into_iter()
        .filter(Volume::external)
        .collect())
}

fn parse_with(output: &str, selection: Selection) -> Result<Vec<Volume>, serde_json::Error> {
    let listing: Listing = serde_json::from_str(output)?;
    let mut volumes = Vec::new();
    for node in &listing.blockdevices {
        collect(node, false, selection, &mut volumes);
    }
    let mut devices = BTreeSet::new();
    volumes.retain(|volume| devices.insert(volume.path.clone()));
    Ok(volumes)
}

fn collect(node: &Node, parent_removable: bool, selection: Selection, volumes: &mut Vec<Volume>) {
    let removable = parent_removable || node.rm || node.hotplug;
    for child in &node.children {
        collect(child, removable, selection, volumes);
    }
    if !eligible(node, selection) {
        return;
    }
    let mountpoint = node
        .mountpoint
        .iter()
        .chain(node.mountpoints.iter().flatten())
        .find(|point| !point.is_empty())
        .cloned();
    volumes.push(Volume {
        path: node.path.clone(),
        size: node.size.clone(),
        label: node.label.clone().unwrap_or_default(),
        filesystem: node.fstype.clone().unwrap_or_default(),
        mountpoint,
        removable,
    });
}

fn eligible(node: &Node, selection: Selection) -> bool {
    let supported = match selection {
        Selection::External => matches!(node.kind.as_str(), "disk" | "part"),
        Selection::Available => matches!(
            node.kind.as_str(),
            "disk" | "part" | "crypt" | "lvm" | "raid0" | "raid1" | "raid5" | "raid6" | "raid10"
        ),
    };
    if !supported || !device_path(&node.path) || node.name.starts_with("zram") {
        return false;
    }
    let Some(format) = node.fstype.as_deref().filter(|format| !format.is_empty()) else {
        return false;
    };
    if format == "swap"
        || selection == Selection::Available
            && matches!(
                format.to_lowercase().as_str(),
                "swap" | "crypto_luks" | "lvm2_member" | "linux_raid_member" | "zfs_member"
            )
    {
        return false;
    }
    !node
        .mountpoint
        .iter()
        .chain(node.mountpoints.iter().flatten())
        .any(|point| protected(point, selection))
}

fn device_path(path: &str) -> bool {
    let path = Path::new(path);
    path.starts_with("/dev")
        && path != Path::new("/dev")
        && path
            .components()
            .all(|part| matches!(part, Component::RootDir | Component::Normal(_)))
}

fn protected(point: &str, selection: Selection) -> bool {
    if selection == Selection::External {
        return matches!(point, "/" | "/boot" | "/home" | "/usr" | "/var") || point.starts_with("/boot/");
    }
    let path = Path::new(point);
    ["/", "/home", "/run", "/tmp"]
        .iter()
        .any(|root| path == Path::new(root))
        || [
            "/boot",
            "/usr",
            "/var",
            "/etc",
            "/dev",
            "/proc",
            "/sys",
            "/run/credentials",
        ]
        .iter()
        .any(|root| path.starts_with(root))
}
