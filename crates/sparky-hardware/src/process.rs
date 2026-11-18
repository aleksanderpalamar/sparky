#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcStat {
    pub pid: u32,
    pub comm: String,
    pub ppid: u32,
    pub ticks: u64,
    pub started: u64,
}

pub fn parse_stat(text: &str) -> Option<ProcStat> {
    let open = text.find('(')?;
    let close = text.rfind(')')?;
    let fields: Vec<&str> = text.get(close + 1..)?.split_whitespace().collect();
    let number = |index: usize| fields.get(index)?.parse::<u64>().ok();
    Some(ProcStat {
        pid: text.get(..open)?.trim().parse().ok()?,
        comm: text.get(open + 1..close)?.to_owned(),
        ppid: fields.get(1)?.parse().ok()?,
        ticks: number(11)? + number(12)?,
        started: number(19).unwrap_or_default(),
    })
}
