use std::collections::HashMap;

use crate::domain::Manager;
use crate::domain::updates::{UpdateItem, UpdateKind, UpdateSource, valid_native_package};

fn item(
    id: &str,
    installed: Option<&str>,
    available: Option<&str>,
    manager: Manager,
) -> Option<UpdateItem> {
    valid_native_package(id).then(|| UpdateItem {
        id: id.to_owned(),
        name: id.to_owned(),
        installed: installed.map(str::to_owned),
        available: available.map(str::to_owned),
        source: UpdateSource::Native(manager),
        kind: UpdateKind::System,
    })
}

pub fn pacman(output: &str) -> Result<Vec<UpdateItem>, String> {
    output
        .lines()
        .map(|line| {
            let (name, versions) = line
                .split_once(' ')
                .ok_or_else(|| format!("Invalid checkupdates row: {line}"))?;
            let (installed, available) = versions
                .split_once(" -> ")
                .ok_or_else(|| format!("Invalid checkupdates versions: {line}"))?;
            item(name, Some(installed), Some(available), Manager::Pacman)
                .ok_or_else(|| format!("Invalid package: {name}"))
        })
        .collect()
}

pub fn apt(output: &str) -> Result<Vec<UpdateItem>, String> {
    output
        .lines()
        .filter_map(|line| line.strip_prefix("Inst "))
        .filter_map(|line| {
            let (name, versions) = match line.split_once(' ') {
                Some(parts) => parts,
                None => return Some(Err(format!("Invalid APT update: {line}"))),
            };
            let installed = versions
                .strip_prefix('[')
                .and_then(|rest| rest.split_once(']'))
                .map(|(version, _)| version);
            let Some(installed) = installed else {
                if versions.starts_with('(') {
                    return None;
                }
                return Some(Err(format!("Invalid APT update: {line}")));
            };
            let available = versions
                .split_once('(')
                .and_then(|(_, rest)| rest.split_whitespace().next())
                .ok_or_else(|| format!("Missing available APT version: {line}"));
            Some(available.and_then(|version| {
                item(name, Some(installed), Some(version), Manager::Apt)
                    .ok_or_else(|| format!("Invalid package: {name}"))
            }))
        })
        .collect()
}

pub fn rpm_versions(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .filter(|(name, _)| valid_native_package(name))
        .map(|(name, version)| (name.to_owned(), version.to_owned()))
        .collect()
}

pub fn dnf(output: &str, installed: &HashMap<String, String>) -> Result<Vec<UpdateItem>, String> {
    let items: Vec<_> = output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let (Some(name), Some(version), Some(_repo), None) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            else {
                return None;
            };
            let current = installed.get(name)?;
            item(name, Some(current), Some(version), Manager::Dnf)
        })
        .collect();
    if !output.trim().is_empty() && items.is_empty() {
        return Err("DNF returned updates without recognized packages".into());
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::{apt, dnf, pacman, rpm_versions};

    #[test]
    fn interpreta_pacman_e_apt_com_versoes() {
        assert_eq!(
            pacman("mesa 1.0-1 -> 1.1-1\n").unwrap()[0]
                .available
                .as_deref(),
            Some("1.1-1")
        );
        assert_eq!(
            apt("Inst libc6:amd64 [2.36-1] (2.37-1 Debian:stable [amd64])\n").unwrap()[0].id,
            "libc6:amd64"
        );
        assert_eq!(
            apt("Inst new-dependency (1.0 Debian:stable [amd64])\n")
                .unwrap()
                .len(),
            0
        );
        assert!(apt("Inst broken package\n").is_err());
        assert!(pacman("broken\n").is_err());
    }

    #[test]
    fn dnf_precisa_corresponder_a_um_pacote_instalado() {
        let installed = rpm_versions("mesa.x86_64\t24.1-1\n");
        let updates = dnf("mesa.x86_64 24.2-1 updates\n", &installed).unwrap();
        assert_eq!(updates[0].installed.as_deref(), Some("24.1-1"));
        assert!(dnf("unrecognized output", &installed).is_err());
    }
}
