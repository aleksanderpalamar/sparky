pub fn name(database: &str, vendor: u16, device: u16) -> String {
    let ids = format!("{vendor:04x}:{device:04x}");
    let mut lines = database.lines();
    let vendor_name = lines
        .by_ref()
        .filter(|line| !line.starts_with(char::is_whitespace))
        .filter_map(entry)
        .find(|(id, _)| *id == vendor)
        .map(|(_, name)| name);
    let Some(vendor_name) = vendor_name else {
        return format!("GPU PCI {ids}");
    };
    let device_name = lines
        .take_while(|line| line.starts_with(['\t', '#']) || line.trim().is_empty())
        .filter_map(|line| line.strip_prefix('\t'))
        .filter_map(entry)
        .find(|(id, _)| *id == device)
        .map(|(_, name)| name);
    match device_name {
        Some(name) => format!("{vendor_name} {name}"),
        None => format!("{vendor_name} [{ids}]"),
    }
}

fn entry(line: &str) -> Option<(u16, &str)> {
    let (id, name) = line.split_once(char::is_whitespace)?;
    let name = name.trim();
    if id.len() != 4 || name.is_empty() {
        return None;
    }
    Some((u16::from_str_radix(id, 16).ok()?, name))
}

#[cfg(test)]
#[path = "pci_tests.rs"]
mod tests;
