pub fn parse_cpu_list(text: &str) -> Option<Vec<u32>> {
    let text = text.trim();
    if text.is_empty() {
        return Some(Vec::new());
    }
    let mut cpus = Vec::new();
    for part in text.split(',') {
        let (first, last) = part.split_once('-').unwrap_or((part, part));
        let (first, last) = (first.parse::<u32>().ok()?, last.parse::<u32>().ok()?);
        if last < first {
            return None;
        }
        cpus.extend(first..=last);
    }
    cpus.sort_unstable();
    cpus.dedup();
    Some(cpus)
}
