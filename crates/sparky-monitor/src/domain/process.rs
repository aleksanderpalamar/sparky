#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcStat {
    pub pid: u32,
    pub comm: String,
    pub ppid: u32,
    pub ticks: u64,
}

const COMM_LIMIT: usize = 15;

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
    })
}

pub fn kernel_thread(stat: &ProcStat) -> bool {
    stat.pid == 2 || stat.ppid == 2
}

pub fn resident_pages(statm: &str) -> Option<u64> {
    statm.split_whitespace().nth(1)?.parse().ok()
}

pub fn io_bytes(io: &str) -> Option<(u64, u64)> {
    let field = |key: &str| {
        io.lines()
            .find_map(|line| line.strip_prefix(key)?.strip_prefix(':'))
            .and_then(|value| value.trim().parse::<u64>().ok())
    };
    Some((field("read_bytes")?, field("write_bytes")?))
}

pub fn real_uid(status: &str) -> Option<u32> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

pub fn program(comm: &str, cmdline: &str) -> String {
    let first = cmdline.split('\0').next().unwrap_or_default();
    match first.rsplit('/').next().filter(|name| !name.is_empty()) {
        Some(name) if !name.contains(' ') => name.to_owned(),
        _ => comm.to_owned(),
    }
}

pub fn display_name(comm: &str, cmdline: &str) -> String {
    let program = program(comm, cmdline);
    match comm.chars().count() >= COMM_LIMIT && program.starts_with(comm) {
        true => program,
        false => comm.to_owned(),
    }
}
