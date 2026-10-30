#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetCounters {
    pub name: String,
    pub received: u64,
    pub sent: u64,
}

pub fn parse_net_dev(text: &str) -> Vec<NetCounters> {
    text.lines()
        .skip(2)
        .filter_map(|line| {
            let (name, values) = line.split_once(':')?;
            let name = name.trim();
            if name == "lo" {
                return None;
            }
            let values: Vec<u64> = values
                .split_whitespace()
                .map(str::parse)
                .collect::<Result<_, _>>()
                .ok()?;
            Some(NetCounters {
                name: name.to_owned(),
                received: *values.first()?,
                sent: *values.get(8)?,
            })
        })
        .collect()
}
